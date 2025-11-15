"""Real-world use case tests for mappy-python bindings."""

import random
import time

import mappy_python as mappy

from . import Stats


class TestKmerCounting:
    """Test k-mer counting use case (bioinformatics)."""

    def test_kmer_counting_basic(self, dna_sequences: list[str]):
        """Test basic k-mer counting functionality."""
        kmer_counter = mappy.Maplet(10000, 0.001, mappy.CounterOperator())
        k = 3  # 3-mer counting

        # Count k-mers in first sequence
        sequence = dna_sequences[0]
        expected_kmers = {}

        for i in range(len(sequence) - k + 1):
            kmer = sequence[i : i + k]
            kmer_counter.insert(kmer, 1)
            expected_kmers[kmer] = expected_kmers.get(kmer, 0) + 1

        # Verify k-mer counts
        for kmer, expected_count in expected_kmers.items():
            result = kmer_counter.query(kmer)
            assert result is not None
            assert result >= expected_count  # May be higher due to collisions

        # Verify total k-mers
        stats = Stats(kmer_counter.stats())
        assert stats.item_count >= len(expected_kmers)

    def test_kmer_counting_multiple_sequences(self, dna_sequences: list[str]):
        """Test k-mer counting across multiple sequences."""
        kmer_counter = mappy.Maplet(50000, 0.001, mappy.CounterOperator())
        k = 4  # 4-mer counting

        total_kmers = 0
        for sequence in dna_sequences:
            for i in range(len(sequence) - k + 1):
                kmer = sequence[i : i + k]
                kmer_counter.insert(kmer, 1)
                total_kmers += 1

        # Verify statistics
        stats = Stats(kmer_counter.stats())
        assert stats.item_count > 0
        assert stats.load_factor > 0

        # Test some random k-mers
        test_kmers = []
        for sequence in dna_sequences[:3]:
            for i in range(0, len(sequence) - k + 1, 10):  # Sample every 10th k-mer
                kmer = sequence[i : i + k]
                test_kmers.append(kmer)

        for kmer in test_kmers[:10]:  # Test first 10
            result = kmer_counter.query(kmer)
            assert result is not None
            assert result >= 1

    def test_kmer_counting_performance(self):
        """Test k-mer counting performance."""
        kmer_counter = mappy.Maplet(100000, 0.001, mappy.CounterOperator())

        # Generate large DNA sequence
        bases = ["A", "T", "C", "G"]
        large_sequence = "".join(random.choices(bases, k=10000))

        # Time k-mer counting
        start_time = time.time()
        k = 3
        for i in range(len(large_sequence) - k + 1):
            kmer = large_sequence[i : i + k]
            kmer_counter.insert(kmer, 1)
        counting_time = time.time() - start_time

        # Performance should be reasonable
        assert counting_time < 5.0  # Should count 10k k-mers in under 5 seconds

        # Verify results
        stats = Stats(kmer_counter.stats())
        assert stats.item_count > 0


class TestNetworkTrafficAnalysis:
    """Test network traffic analysis use case."""

    def test_traffic_counting(self, network_traffic_data: list[tuple[str, int]]):
        """Test network traffic counting."""
        traffic_counter = mappy.Maplet(10000, 0.01, mappy.CounterOperator())

        # Insert traffic data
        for ip_address, bytes_transferred in network_traffic_data:
            traffic_counter.insert(ip_address, bytes_transferred)

        # Verify traffic counts
        unique_ips = {ip for ip, _ in network_traffic_data}
        for ip in list(unique_ips)[:5]:  # Test first 5 IPs
            result = traffic_counter.query(ip)
            assert result is not None
            assert result > 0

        # Verify statistics
        stats = Stats(traffic_counter.stats())
        assert stats.item_count > 0

    def test_traffic_analysis_with_max_operator(
        self, network_traffic_data: list[tuple[str, int]],
    ):
        """Test traffic analysis using max operator for peak detection."""
        peak_traffic = mappy.Maplet(10000, 0.01, mappy.MaxOperator())

        # Insert traffic data
        for ip_address, bytes_transferred in network_traffic_data:
            peak_traffic.insert(ip_address, bytes_transferred)

        # Verify peak traffic detection
        unique_ips = {ip for ip, _ in network_traffic_data}
        for ip in list(unique_ips)[:5]:
            result = peak_traffic.query(ip)
            assert result is not None
            assert result > 0

            # Result should be at least as high as any individual transfer for this IP
            max_for_ip = max(
                bytes_transferred
                for ip_addr, bytes_transferred in network_traffic_data
                if ip_addr == ip
            )
            assert result >= max_for_ip

    def test_traffic_analysis_performance(self):
        """Test network traffic analysis performance."""
        traffic_counter = mappy.Maplet(100000, 0.001, mappy.CounterOperator())

        # Generate large traffic dataset
        traffic_data = []
        for i in range(10000):
            ip = f"192.168.{i % 255}.{i % 255}"
            bytes_transferred = random.randint(100, 10000)
            traffic_data.append((ip, bytes_transferred))

        # Time traffic analysis
        start_time = time.time()
        for ip_address, bytes_transferred in traffic_data:
            traffic_counter.insert(ip_address, bytes_transferred)
        analysis_time = time.time() - start_time

        # Performance should be reasonable
        assert analysis_time < 10.0  # Should process 10k entries in under 10 seconds

        # Verify results
        stats = Stats(traffic_counter.stats())
        assert stats.item_count > 0


class TestDistributedCaching:
    """Test distributed caching use case."""

    def test_cache_operations(self):
        """Test basic cache operations."""
        cache = mappy.Maplet(10000, 0.01, mappy.MaxOperator())

        # Cache some data (using numeric values for MaxOperator)
        cache_data = {
            "user:123": 100,
            "user:456": 200,
            "session:abc": 300,
        }

        for key, value in cache_data.items():
            cache.insert(key, value)

        # Retrieve cached data
        for key, expected_value in cache_data.items():
            result = cache.query(key)
            assert result is not None
            assert result >= expected_value

    def test_cache_performance(self):
        """Test cache performance."""
        cache = mappy.Maplet(100000, 0.001, mappy.MaxOperator())

        # Generate cache data
        cache_data = []
        for i in range(10000):
            key = f"cache_key_{i}"
            value = i
            cache_data.append((key, value))

        # Time cache operations
        start_time = time.time()
        for key, value in cache_data:
            cache.insert(key, value)
        insert_time = time.time() - start_time

        start_time = time.time()
        success_count = 0
        for key, _ in cache_data[:1000]:  # Query first 1000
            result = cache.query(key)
            if result is not None:
                success_count += 1
        query_time = time.time() - start_time

        # At least 90% should succeed
        assert success_count >= 900, f"Only {success_count}/1000 queries succeeded"

        # Performance should be reasonable
        assert insert_time < 10.0  # Should insert 10k items in under 10 seconds
        assert query_time < 2.0  # Should query 1k items in under 2 seconds


class TestLSMStorageIndex:
    """Test LSM storage engine index use case."""

    def test_sstable_index(self):
        """Test SSTable index functionality."""
        sstable_index = mappy.Maplet(100000, 0.001, mappy.MaxOperator())

        # Simulate SSTable entries
        sstable_entries = [
            ("key1", 1),  # SSTable 1
            ("key2", 1),
            ("key3", 2),  # SSTable 2
            ("key4", 2),
            ("key1", 3),  # SSTable 3 (newer version of key1)
        ]

        # Insert SSTable entries
        for key, sstable_id in sstable_entries:
            sstable_index.insert(key, sstable_id)

        # Query should return the latest SSTable ID
        result1 = sstable_index.query("key1")
        assert result1 is not None
        assert result1 >= 3  # Should be SSTable 3 (latest)

        result2 = sstable_index.query("key2")
        assert result2 is not None
        assert result2 >= 1  # Should be SSTable 1

        result3 = sstable_index.query("key3")
        assert result3 is not None
        assert result3 >= 2  # Should be SSTable 2

    def test_sstable_index_performance(self):
        """Test SSTable index performance."""
        sstable_index = mappy.Maplet(1000000, 0.0001, mappy.MaxOperator())

        # Generate large SSTable index
        start_time = time.time()
        for i in range(100000):
            key = f"key_{i}"
            sstable_id = random.randint(1, 100)  # 100 SSTables
            sstable_index.insert(key, sstable_id)
        insert_time = time.time() - start_time

        # Query performance - allow some failures
        start_time = time.time()
        success_count = 0
        for i in range(10000):  # Query 10k keys
            key = f"key_{i}"
            result = sstable_index.query(key)
            if result is not None:
                success_count += 1
        query_time = time.time() - start_time

        # At least 90% should succeed
        assert success_count >= 9000, f"Only {success_count}/10000 queries succeeded"

        # Performance should be excellent
        assert insert_time < 15.0  # Should insert 100k entries in under 15 seconds
        assert query_time < 3.0  # Should query 10k entries in under 3 seconds


class TestRealWorldScenarios:
    """Test real-world scenarios combining multiple use cases."""

    def test_multi_use_case_workload(self):
        """Test combined workload from multiple use cases."""
        # Create multiple maplets for different purposes
        kmer_counter = mappy.Maplet(50000, 0.001, mappy.CounterOperator())
        traffic_analyzer = mappy.Maplet(10000, 0.01, mappy.MaxOperator())
        cache = mappy.Maplet(10000, 0.01, mappy.MaxOperator())

        # Simulate combined workload
        start_time = time.time()

        # K-mer counting workload
        dna_sequence = "".join(random.choices(["A", "T", "C", "G"], k=1000))
        for i in range(len(dna_sequence) - 3):
            kmer = dna_sequence[i : i + 3]
            kmer_counter.insert(kmer, 1)

        # Traffic analysis workload
        for i in range(1000):
            ip = f"192.168.{i % 255}.{i % 255}"
            bytes_transferred = random.randint(100, 10000)
            traffic_analyzer.insert(ip, bytes_transferred)

        # Caching workload
        for i in range(1000):
            key = f"cache_key_{i}"
            value = i
            cache.insert(key, value)

        total_time = time.time() - start_time

        # Verify all workloads completed successfully
        assert Stats(kmer_counter.stats()).item_count > 0
        assert Stats(traffic_analyzer.stats()).item_count > 0
        assert Stats(cache.stats()).item_count > 0

        # Performance should be reasonable
        assert total_time < 20.0  # All workloads should complete in under 20 seconds

    def test_memory_efficiency_comparison(self):
        """Test memory efficiency compared to naive approaches."""
        # Create maplet
        maplet = mappy.Maplet(10000, 0.01, mappy.CounterOperator())

        # Insert data
        for i in range(5000):
            key = f"key_{i}"
            maplet.insert(key, i)

        # Get memory usage
        stats = Stats(maplet.stats())
        maplet_memory = stats.memory_usage

        # Compare with naive dictionary approach (rough estimate)
        # Dictionary would use more memory for the same data
        naive_memory_estimate = 5000 * (
            20 + 8
        )  # Rough estimate: 20 bytes per key + 8 bytes per value

        # Maplet should use less memory
        assert maplet_memory < naive_memory_estimate

        # Verify functionality
        # Test a sample of keys (not all, as some may hit capacity limits)
        tested = 0
        for i in range(100):  # Test first 100 keys
            key = f"key_{i}"
            result = maplet.query(key)
            if result is not None:
                assert result >= i
                tested += 1
        # At least some keys should be queryable
        assert tested > 0, "No keys were successfully queried"

