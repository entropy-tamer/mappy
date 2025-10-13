"""
Mappy Python Bindings

Space-efficient maplet data structures for approximate key-value mappings.
"""

from .mappy_python import (
    Maplet,
    CounterOperator,
    MaxOperator,
    MinOperator,
    SetOperator,
    VectorOperator,
    CustomOperator,
    MapletError,
    MapletStats,
)

__version__ = "0.1.1"
__author__ = "Reynard Team"
__email__ = "team@reynard.dev"

__all__ = [
    "Maplet",
    "CounterOperator",
    "MaxOperator", 
    "MinOperator",
    "SetOperator",
    "VectorOperator",
    "CustomOperator",
    "MapletError",
    "MapletStats",
]
