"""ML task implementations for mappy DataLoader evaluation."""

try:
    from .image_captioning import ImageCaptioningTask
    from .image_classification import ImageClassificationTask
    from .image_diffusion import ImageDiffusionTask
    from .nlp_tasks import NLPTask
    from .recommendation import RecommendationTask

    __all__ = [
        "ImageCaptioningTask",
        "ImageClassificationTask",
        "ImageDiffusionTask",
        "NLPTask",
        "RecommendationTask",
    ]
except ImportError:
    # Handle case where dependencies might not be available
    __all__ = []

