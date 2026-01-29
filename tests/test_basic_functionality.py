"""Basic functionality tests for mappy-python bindings."""


import mappy_python as mappy
import pytest

from . import Stats


class TestBasicMapletOperations:
    """Test basic maplet operations."""

    def test_maplet_creation(self):
        """Test creating maplets with different operators."""
        # Counter operator
        counter_maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())
        assert counter_maplet is not None

        # Max operator
        max_maplet = mappy.Maplet(1000, 0.01, mappy.MaxOperator())
        assert max_maplet is not None

        # Min operator
        min_maplet = mappy.Maplet(1000, 0.01, mappy.MinOperator())
        assert min_maplet is not None

    def test_insert_and_query_strings(self, sample_strings: list[str]):
        """Test inserting and querying string keys."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert string keys with integer values
        for i, key in enumerate(sample_strings[:10]):
            maplet.insert(key, i + 1)

        # Query inserted keys
        for i, key in enumerate(sample_strings[:10]):
            result = maplet.query(key)
            assert result is not None
            assert result >= i + 1  # May be higher due to collisions

    def test_insert_and_query_integers(self, sample_integers: list[int]):
        """Test inserting and querying with integer keys and values."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert integer keys with integer values
        for i, value in enumerate(sample_integers[:10]):
            key = f"int_key_{i}"
            maplet.insert(key, value)

        # Query inserted keys
        for i, expected_value in enumerate(sample_integers[:10]):
            key = f"int_key_{i}"
            result = maplet.query(key)
            assert result is not None
            assert result >= expected_value

    def test_contains_operation(self, sample_strings: list[str]):
        """Test contains operation."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert some keys
        inserted_keys = sample_strings[:5]
        for key in inserted_keys:
            maplet.insert(key, 1)

        # Test contains for inserted keys
        for key in inserted_keys:
            assert maplet.contains(key) is True

        # Test contains for non-inserted keys
        non_inserted_keys = sample_strings[5:10]
        for key in non_inserted_keys:
            # Contains may return True due to false positives, but should be consistent
            result = maplet.contains(key)
            assert isinstance(result, bool)

    def test_delete_operation(self, sample_strings: list[str]):
        """Test delete operation."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert keys
        keys_to_delete = sample_strings[:3]
        for key in keys_to_delete:
            maplet.insert(key, 10)

        # Verify keys exist
        for key in keys_to_delete:
            assert maplet.contains(key) is True

        # Delete keys
        for key in keys_to_delete:
            maplet.delete(key)

        # Verify keys are deleted (may still show false positives)
        for key in keys_to_delete:
            result = maplet.query(key)
            # After deletion, result should be None or 0
            assert result is None or result == 0

    def test_stats_operation(self, sample_strings: list[str]):
        """Test statistics operation."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert some data
        for i, key in enumerate(sample_strings[:10]):
            maplet.insert(key, i + 1)

        # Get statistics
        stats_dict = maplet.stats()
        stats = Stats(stats_dict)

        # Verify stats structure
        assert hasattr(stats, "load_factor")
        assert hasattr(stats, "memory_usage")
        assert hasattr(stats, "item_count")
        assert hasattr(stats, "false_positive_rate")

        # Verify stats values
        assert 0.0 <= stats.load_factor <= 1.0
        assert stats.memory_usage > 0
        assert stats.item_count >= 0
        assert 0.0 <= stats.false_positive_rate <= 1.0

    def test_clear_operation(self, sample_strings: list[str]):
        """Test clear operation."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert data
        for key in sample_strings[:10]:
            maplet.insert(key, 1)

        # Verify data exists
        stats = Stats(maplet.stats())
        assert stats.item_count > 0

        # Clear maplet - Maplet doesn't support clear(), only Engine does
        # So we expect a NotImplementedError
        with pytest.raises(NotImplementedError):
            maplet.clear()

    def test_capacity_and_resize(self):
        """Test maplet capacity and resize functionality."""
        # Create small maplet
        maplet = mappy.Maplet(100, 0.01, mappy.CounterOperator())

        initial_stats = Stats(maplet.stats())
        initial_capacity = initial_stats.memory_usage

        # Insert many items to trigger resize
        for i in range(200):
            maplet.insert(f"key_{i}", i)

        final_stats = Stats(maplet.stats())
        final_capacity = final_stats.memory_usage

        # Capacity should have increased
        assert final_capacity >= initial_capacity
        assert final_stats.item_count >= 100


class TestMergeOperators:
    """Test different merge operators."""

    def test_counter_operator(self):
        """Test counter operator behavior."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert same key multiple times
        key = "test_key"
        maplet.insert(key, 5)
        maplet.insert(key, 3)
        maplet.insert(key, 2)

        # Result should be sum of all values
        result = maplet.query(key)
        assert result is not None
        assert result >= 10  # 5 + 3 + 2, may be higher due to collisions

    def test_max_operator(self):
        """Test max operator behavior."""
        maplet = mappy.Maplet(1000, 0.01, mappy.MaxOperator())

        # Insert same key with different values
        key = "test_key"
        maplet.insert(key, 5)
        maplet.insert(key, 10)
        maplet.insert(key, 3)

        # Result should be maximum value
        result = maplet.query(key)
        assert result is not None
        assert result >= 10  # Max of 5, 10, 3

    def test_min_operator(self):
        """Test min operator behavior."""
        maplet = mappy.Maplet(1000, 0.01, mappy.MinOperator())

        # Insert same key with different values
        key = "test_key"
        maplet.insert(key, 5)
        maplet.insert(key, 10)
        maplet.insert(key, 3)

        # Result should be minimum value
        result = maplet.query(key)
        assert result is not None
        assert result <= 3  # Min of 5, 10, 3

    def test_custom_operator(self):
        """Test custom operator behavior."""
        # Note: CustomOperator may not be implemented yet
        # Skip this test if CustomOperator is not available
        try:
            def custom_merge(a, b):
                return a + b * 2

            maplet = mappy.Maplet(1000, 0.01, mappy.CustomOperator(custom_merge))

            # Insert same key multiple times
            key = "test_key"
            maplet.insert(key, 5)
            maplet.insert(key, 3)

            # Result should follow custom merge logic
            result = maplet.query(key)
            assert result is not None
            # Custom logic: 5 + (3 * 2) = 11, but may be higher due to collisions
            assert result >= 11
        except (AttributeError, TypeError):
            pytest.skip("CustomOperator not yet implemented")


class TestErrorHandling:
    """Test error handling and edge cases."""

    def test_invalid_parameters(self):
        """Test maplet creation with invalid parameters."""
        # Test with zero capacity
        with pytest.raises((ValueError, RuntimeError)):
            mappy.Maplet(0, 0.01, mappy.CounterOperator())

        # Test with invalid false positive rate
        with pytest.raises((ValueError, RuntimeError)):
            mappy.Maplet(1000, 0.0, mappy.CounterOperator())

        with pytest.raises((ValueError, RuntimeError)):
            mappy.Maplet(1000, 1.0, mappy.CounterOperator())

    def test_none_values(self):
        """Test handling of None values."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Test inserting None key
        with pytest.raises((TypeError, ValueError)):
            maplet.insert(None, 1)

        # Test querying None key
        with pytest.raises((TypeError, ValueError)):
            maplet.query(None)

    def test_empty_string_handling(self):
        """Test handling of empty strings."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert empty string
        maplet.insert("", 1)

        # Query empty string
        result = maplet.query("")
        assert result is not None
        assert result >= 1

    def test_very_large_values(self):
        """Test handling of very large values."""
        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())

        # Insert very large value
        large_value = 2**63 - 1
        maplet.insert("large_key", large_value)

        # Query large value
        result = maplet.query("large_key")
        assert result is not None
        assert result >= large_value


class TestConcurrency:
    """Test concurrent access to maplets."""

    def test_thread_safety(self, sample_strings: list[str]):
        """Test thread safety of maplet operations."""
        import threading
        import time

        maplet = mappy.Maplet(1000, 0.01, mappy.CounterOperator())
        results = []
        errors = []

        def worker(worker_id: int):
            try:
                for i in range(10):
                    key = f"worker_{worker_id}_item_{i}"
                    maplet.insert(key, 1)
                    time.sleep(
                        0.001,
                    )  # Small delay to increase chance of race conditions
                results.append(f"worker_{worker_id}_completed")
            except Exception as e:
                errors.append(f"worker_{worker_id}_error: {e}")

        # Create multiple threads
        threads = []
        for i in range(4):
            thread = threading.Thread(target=worker, args=(i,))
            threads.append(thread)
            thread.start()

        # Wait for all threads to complete
        for thread in threads:
            thread.join()

        # Verify no errors occurred
        assert len(errors) == 0, f"Errors occurred: {errors}"

        # Verify all workers completed
        assert len(results) == 4

        # Verify data integrity
        stats = Stats(maplet.stats())
        assert stats.item_count >= 40  # 4 workers * 10 items each

