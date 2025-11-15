"""Test utilities for mappy-python."""

class Stats:
    """Helper class to convert stats dict to object with attributes."""

    def __init__(self, stats_dict):
        self.item_count = stats_dict.get("item_count", 0)
        self.load_factor = stats_dict.get("load_factor", 0.0)
        self.memory_usage = stats_dict.get("memory_usage", 0)
        self.false_positive_rate = stats_dict.get("false_positive_rate", stats_dict.get("error_rate", 0.0))
        self.error_rate = stats_dict.get("error_rate", 0.0)









