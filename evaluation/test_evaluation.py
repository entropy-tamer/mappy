"""Test script to evaluate the evaluation suite."""

import sys
import traceback
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

def test_imports():
    """Test that all modules can be imported."""
    try:
        import mappy_python
    except ImportError:
        return False

    try:
        pass
    except Exception:
        traceback.print_exc()
        return False

    try:
        pass
    except Exception:
        traceback.print_exc()
        return False

    try:
        pass
    except Exception:
        traceback.print_exc()
        return False

    try:
        pass
    except Exception:
        traceback.print_exc()
        return False

    try:
        pass
    except Exception:
        traceback.print_exc()
        return False

    return True


def test_mappy_api():
    """Test mappy API usage."""
    try:
        import mappy_python
        import numpy as np

        # Test Maplet with VectorOperator
        maplet = mappy_python.Maplet(
            capacity=100,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator(),
        )

        # Test storing a list
        test_data = [1.0, 2.0, 3.0, 4.0, 5.0]
        maplet.insert("test_key", test_data)

        # Test querying
        result = maplet.query("test_key")
        if result is not None:
            pass
        else:
            return False

        # Test with numpy array
        arr = np.array([1.0, 2.0, 3.0])
        maplet.insert("numpy_key", arr)
        result = maplet.query("numpy_key")
        if result is not None:
            pass
        else:
            return False

        return True
    except Exception:
        traceback.print_exc()
        return False


def test_dataloader_basic():
    """Test basic dataloader functionality."""
    try:
        import mappy_python

        from evaluation.dataloader import MappyDataset, store_data_in_mappy

        # Create maplet
        maplet = mappy_python.Maplet(
            capacity=100,
            false_positive_rate=0.01,
            operator=mappy_python.VectorOperator(),
        )

        # Store some test data
        data_dict = {
            "item_0": [1.0, 2.0, 3.0],
            "item_1": [4.0, 5.0, 6.0],
            "item_2": [7.0, 8.0, 9.0],
        }
        keys = store_data_in_mappy(maplet, data_dict)

        # Create dataset
        dataset = MappyDataset(
            maplet=maplet,
            keys=keys,
            cache_data=False,  # Don't cache for quick test
        )

        # Test getting an item
        dataset[0]

        return True
    except Exception:
        traceback.print_exc()
        return False


def test_task_creation():
    """Test that tasks can be created."""
    try:
        from evaluation.tasks.image_classification import ImageClassificationTask

        task = ImageClassificationTask(
            dataset_name="synthetic",  # Use synthetic to avoid downloading
            batch_size=8,
            num_epochs=1,
            use_mappy=True,
            cache_data=False,
        )

        # Test that maplet was created
        if task.maplet is not None:
            pass
        else:
            return False

        return True
    except Exception:
        traceback.print_exc()
        return False


def main():
    """Run all tests."""
    results = []

    results.append(("Imports", test_imports()))
    results.append(("Mappy API", test_mappy_api()))
    results.append(("DataLoader", test_dataloader_basic()))
    results.append(("Task Creation", test_task_creation()))


    passed = sum(1 for _, result in results if result)
    total = len(results)

    for _name, _result in results:
        pass


    if passed == total:
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())

