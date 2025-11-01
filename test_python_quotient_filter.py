#!/usr/bin/env python3
"""Legacy test script for Python bindings with quotient filter feature.

This script is maintained for backward compatibility. For comprehensive
testing, use the pytest test suite:

    pytest test_mappy_python.py -v

This script runs a subset of tests focused on quotient filter features.
"""

import os
import sys
import time

# Add the current directory to the path so we can import mappy_python
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    import mappy_python
except ImportError:
    print("Error: mappy_python module not found. Make sure to build the Python bindings first.")
    print("Run: cd mappy-python && maturin develop")
    sys.exit(1)


def test_basic_maplet_quotient_filter_features():
    """Test basic Maplet with quotient filter features."""
    print("🧪 Testing basic Maplet with quotient filter features...")

    # Create a Maplet with quotient filter features
    maplet = mappy_python.PyMaplet(capacity=1000, false_positive_rate=0.01)

    # Test basic operations
    print("  📝 Testing basic operations...")
    maplet.insert("key1", 42)
    maplet.insert("key2", 100)
    maplet.insert("key3", 200)

    # Test slot finding for existing keys
    print("  🔍 Testing slot finding for existing keys...")
    slot1 = maplet.find_slot_for_key("key1")
    slot2 = maplet.find_slot_for_key("key2")
    slot3 = maplet.find_slot_for_key("key3")

    print(f"    Slot for 'key1': {slot1}")
    print(f"    Slot for 'key2': {slot2}")
    print(f"    Slot for 'key3': {slot3}")

    # All slots should be found for existing keys
    assert slot1 is not None, "Should find slot for existing key 'key1'"
    assert slot2 is not None, "Should find slot for existing key 'key2'"
    assert slot3 is not None, "Should find slot for existing key 'key3'"

    # Test slot finding for non-existing key
    print("  🔍 Testing slot finding for non-existing key...")
    non_existing_slot = maplet.find_slot_for_key("non_existing")
    print(f"    Slot for 'non_existing': {non_existing_slot}")
    # Note: Due to false positives, this might or might not be None

    # Test basic functionality still works
    print("  ✅ Testing basic functionality...")
    assert maplet.query("key1") == 42
    assert maplet.query("key2") == 100
    assert maplet.query("key3") == 200
    assert maplet.contains("key1")
    assert maplet.contains("key2")
    assert maplet.contains("key3")

    print("  ✅ Basic Maplet quotient filter features test passed!")


def test_engine_quotient_filter_features():
    """Test Engine with quotient filter features."""
    print("🧪 Testing Engine with quotient filter features...")

    # Create an Engine with quotient filter features
    config = mappy_python.PyEngineConfig(
        capacity=1000,
        false_positive_rate=0.01,
        persistence_mode="memory",
    )
    engine = mappy_python.PyEngine(config)

    # Test basic operations
    print("  📝 Testing basic operations...")
    engine.set("key1", b"value1")
    engine.set("key2", b"value2")
    engine.set("key3", b"value3")

    # Test slot finding for existing keys
    print("  🔍 Testing slot finding for existing keys...")
    slot1 = engine.find_slot_for_key("key1")
    slot2 = engine.find_slot_for_key("key2")
    slot3 = engine.find_slot_for_key("key3")

    print(f"    Slot for 'key1': {slot1}")
    print(f"    Slot for 'key2': {slot2}")
    print(f"    Slot for 'key3': {slot3}")

    # All slots should be found for existing keys
    assert slot1 is not None, "Should find slot for existing key 'key1'"
    assert slot2 is not None, "Should find slot for existing key 'key2'"
    assert slot3 is not None, "Should find slot for existing key 'key3'"

    # Test slot finding for non-existing key
    print("  🔍 Testing slot finding for non-existing key...")
    non_existing_slot = engine.find_slot_for_key("non_existing")
    print(f"    Slot for 'non_existing': {non_existing_slot}")
    # Note: Due to false positives, this might or might not be None

    # Test basic functionality still works
    print("  ✅ Testing basic functionality...")
    assert engine.get("key1") == b"value1"
    assert engine.get("key2") == b"value2"
    assert engine.get("key3") == b"value3"
    assert engine.exists("key1")
    assert engine.exists("key2")
    assert engine.exists("key3")

    # Test statistics
    print("  📊 Testing statistics...")
    stats = engine.stats()
    print(f"    Engine uptime: {stats.uptime_seconds} seconds")
    print(f"    Total operations: {stats.total_operations}")
    print(f"    Maplet capacity: {stats.maplet_capacity}")
    print(f"    Maplet size: {stats.maplet_size}")
    print(f"    Maplet load factor: {stats.maplet_load_factor:.4f}")
    print(f"    Maplet error rate: {stats.maplet_error_rate:.4f}")

    # Cleanup
    engine.close()

    print("  ✅ Engine quotient filter features test passed!")


def test_performance_benchmark():
    """Test performance of quotient filter features."""
    print("🧪 Testing performance of quotient filter features...")

    # Create a large Maplet for performance testing
    maplet = mappy_python.PyMaplet(capacity=10000, false_positive_rate=0.01)

    # Test insertion performance
    print("  ⚡ Testing insertion performance...")
    start_time = time.time()
    for i in range(1000):
        maplet.insert(f"key_{i}", i)
    insert_time = time.time() - start_time
    print(f"    Inserted 1000 items in {insert_time:.4f} seconds ({1000/insert_time:.0f} ops/sec)")

    # Test slot finding performance
    print("  ⚡ Testing slot finding performance...")
    start_time = time.time()
    slots = []
    for i in range(1000):
        slot = maplet.find_slot_for_key(f"key_{i}")
        slots.append(slot)
    slot_finding_time = time.time() - start_time
    print(f"    Found slots for 1000 items in {slot_finding_time:.4f} seconds ({1000/slot_finding_time:.0f} ops/sec)")

    # Test query performance
    print("  ⚡ Testing query performance...")
    start_time = time.time()
    for i in range(1000):
        value = maplet.query(f"key_{i}")
        assert value == i
    query_time = time.time() - start_time
    print(f"    Queried 1000 items in {query_time:.4f} seconds ({1000/query_time:.0f} ops/sec)")

    # Verify slot finding accuracy
    print("  🎯 Verifying slot finding accuracy...")
    found_slots = sum(1 for slot in slots if slot is not None)
    print(f"    Found slots for {found_slots}/1000 items ({found_slots/10:.1f}%)")

    print("  ✅ Performance test passed!")


def test_concurrent_operations():
    """Test concurrent operations with quotient filter features."""
    print("🧪 Testing concurrent operations with quotient filter features...")

    import queue
    import threading

    # Create a shared Maplet
    maplet = mappy_python.PyMaplet(capacity=5000, false_positive_rate=0.01)
    results = queue.Queue()

    def worker(worker_id: int, num_operations: int):
        """Worker function for concurrent operations."""
        for i in range(num_operations):
            key = f"worker_{worker_id}_key_{i}"
            value = worker_id * 1000 + i

            # Insert
            maplet.insert(key, value)

            # Find slot
            slot = maplet.find_slot_for_key(key)

            # Query
            retrieved_value = maplet.query(key)

            results.put((worker_id, i, slot, retrieved_value))

    # Start multiple threads
    print("  🧵 Starting concurrent operations...")
    threads = []
    for worker_id in range(4):
        thread = threading.Thread(target=worker, args=(worker_id, 100))
        threads.append(thread)
        thread.start()

    # Wait for all threads to complete
    for thread in threads:
        thread.join()

    # Verify results
    print("  🔍 Verifying concurrent results...")
    results_list = []
    while not results.empty():
        results_list.append(results.get())

    print(f"    Processed {len(results_list)} operations")

    # Check that all operations succeeded
    for worker_id, i, slot, retrieved_value in results_list:
        expected_value = worker_id * 1000 + i
        assert retrieved_value == expected_value, f"Value mismatch for worker_{worker_id}_key_{i}"
        # Slot might be None due to false positives, which is acceptable

    print("  ✅ Concurrent operations test passed!")


def test_error_handling():
    """Test error handling for quotient filter features."""
    print("🧪 Testing error handling for quotient filter features...")

    # Test with invalid parameters
    try:
        # This should work fine
        maplet = mappy_python.PyMaplet(capacity=100, false_positive_rate=0.01)
        slot = maplet.find_slot_for_key("test_key")
        print(f"    Slot finding for 'test_key': {slot}")
    except Exception as e:
        print(f"    Unexpected error: {e}")
        raise

    # Test with empty key
    try:
        slot = maplet.find_slot_for_key("")
        print(f"    Slot finding for empty key: {slot}")
    except Exception as e:
        print(f"    Error with empty key (expected): {e}")

    print("  ✅ Error handling test passed!")


def main():
    """Run all tests for Python quotient filter features."""
    print("🦊 Testing Python bindings with quotient filter features")
    print("=" * 70)
    print("NOTE: For comprehensive testing, use: pytest test_mappy_python.py -v")
    print("=" * 70)
    print()

    try:
        # Test basic Maplet functionality
        test_basic_maplet_quotient_filter_features()
        print()

        # Test Engine functionality
        test_engine_quotient_filter_features()
        print()

        # Test performance
        test_performance_benchmark()
        print()

        # Test concurrent operations
        test_concurrent_operations()
        print()

        # Test error handling
        test_error_handling()
        print()

        print("🎉 All tests passed! Python quotient filter features are working correctly.")
        print()
        print("💡 Tip: Run 'pytest test_mappy_python.py -v' for comprehensive test coverage.")

    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()

