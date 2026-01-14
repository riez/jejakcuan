"""Dynamic Time Warping for pattern matching in stock data."""

from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any

import numpy as np
from numpy.typing import NDArray


class PatternType(str, Enum):
    """Types of chart patterns."""

    HEAD_SHOULDERS = "head_and_shoulders"
    INVERSE_HEAD_SHOULDERS = "inverse_head_and_shoulders"
    DOUBLE_TOP = "double_top"
    DOUBLE_BOTTOM = "double_bottom"
    TRIPLE_TOP = "triple_top"
    TRIPLE_BOTTOM = "triple_bottom"
    CUP_HANDLE = "cup_and_handle"
    ASCENDING_TRIANGLE = "ascending_triangle"
    DESCENDING_TRIANGLE = "descending_triangle"
    WEDGE_UP = "wedge_up"
    WEDGE_DOWN = "wedge_down"
    FLAG_BULLISH = "flag_bullish"
    FLAG_BEARISH = "flag_bearish"
    ACCUMULATION = "accumulation"
    DISTRIBUTION = "distribution"
    CUSTOM = "custom"


@dataclass
class PatternMatch:
    """Result of pattern matching."""

    pattern_type: PatternType
    similarity: float  # 0-1, higher is better
    start_idx: int
    end_idx: int
    dtw_distance: float
    description: str
    expected_outcome: str  # "bullish", "bearish", "neutral"


@dataclass
class Pattern:
    """A reference pattern for matching."""

    name: str
    pattern_type: PatternType
    template: NDArray[Any]  # Normalized price sequence
    expected_outcome: str
    description: str


class PatternLibrary:
    """Library of reference patterns."""

    def __init__(self) -> None:
        """Initialize library with common patterns."""
        self.patterns: list[Pattern] = []
        self._load_common_patterns()

    def _load_common_patterns(self) -> None:
        """Load common chart patterns."""
        # Double Bottom - W shape
        double_bottom = np.array(
            [1.0, 0.9, 0.8, 0.7, 0.5, 0.3, 0.1, 0.0, 0.2, 0.4, 0.5, 0.4, 0.2, 0.0, 0.1, 0.3, 0.5, 0.7, 0.9, 1.0]
        )
        self.patterns.append(
            Pattern(
                name="Double Bottom",
                pattern_type=PatternType.DOUBLE_BOTTOM,
                template=double_bottom,
                expected_outcome="bullish",
                description="W-shaped reversal pattern indicating potential uptrend",
            )
        )

        # Double Top - M shape
        double_top = np.array(
            [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 0.9, 1.0, 0.8, 0.6, 0.5, 0.6, 0.8, 1.0, 0.9, 0.7, 0.5, 0.3, 0.1, 0.0]
        )
        self.patterns.append(
            Pattern(
                name="Double Top",
                pattern_type=PatternType.DOUBLE_TOP,
                template=double_top,
                expected_outcome="bearish",
                description="M-shaped reversal pattern indicating potential downtrend",
            )
        )

        # Head and Shoulders
        head_shoulders = np.array(
            [0.0, 0.2, 0.4, 0.6, 0.5, 0.3, 0.5, 0.7, 0.9, 1.0, 0.9, 0.7, 0.5, 0.3, 0.5, 0.6, 0.4, 0.2, 0.0]
        )
        self.patterns.append(
            Pattern(
                name="Head and Shoulders",
                pattern_type=PatternType.HEAD_SHOULDERS,
                template=head_shoulders,
                expected_outcome="bearish",
                description="Three-peak pattern with higher middle peak",
            )
        )

        # Inverse Head and Shoulders
        inv_head_shoulders = 1.0 - head_shoulders
        self.patterns.append(
            Pattern(
                name="Inverse Head and Shoulders",
                pattern_type=PatternType.INVERSE_HEAD_SHOULDERS,
                template=inv_head_shoulders,
                expected_outcome="bullish",
                description="Three-trough pattern with lower middle trough",
            )
        )

        # Cup and Handle (normalized 0-1)
        cup_handle = np.array(
            [1.0, 0.9, 0.7, 0.5, 0.3, 0.1, 0.0, 0.1, 0.3, 0.5, 0.7, 0.9, 1.0, 0.95, 0.9, 0.95, 1.0]
        )
        self.patterns.append(
            Pattern(
                name="Cup and Handle",
                pattern_type=PatternType.CUP_HANDLE,
                template=cup_handle,
                expected_outcome="bullish",
                description="U-shaped pattern with small consolidation handle",
            )
        )

        # Ascending Triangle
        ascending = np.array(
            [0.0, 0.2, 0.5, 0.4, 0.3, 0.6, 0.5, 0.4, 0.7, 0.6, 0.5, 0.8, 0.7, 0.6, 0.9, 0.8, 0.7, 1.0]
        )
        self.patterns.append(
            Pattern(
                name="Ascending Triangle",
                pattern_type=PatternType.ASCENDING_TRIANGLE,
                template=ascending,
                expected_outcome="bullish",
                description="Higher lows with flat resistance",
            )
        )

        # Descending Triangle
        descending = 1.0 - ascending
        self.patterns.append(
            Pattern(
                name="Descending Triangle",
                pattern_type=PatternType.DESCENDING_TRIANGLE,
                template=descending,
                expected_outcome="bearish",
                description="Lower highs with flat support",
            )
        )

        # Accumulation (Range with volume breakout prep)
        accumulation = np.array(
            [0.5, 0.55, 0.45, 0.52, 0.48, 0.53, 0.47, 0.54, 0.46, 0.55, 0.45, 0.56, 0.44, 0.57, 0.43, 0.58, 0.6, 0.7, 0.8]
        )
        self.patterns.append(
            Pattern(
                name="Accumulation",
                pattern_type=PatternType.ACCUMULATION,
                template=accumulation,
                expected_outcome="bullish",
                description="Range-bound trading followed by breakout",
            )
        )

    def add_pattern(self, pattern: Pattern) -> None:
        """Add a custom pattern to the library."""
        self.patterns.append(pattern)

    def get_patterns_by_type(self, pattern_type: PatternType) -> list[Pattern]:
        """Get all patterns of a specific type."""
        return [p for p in self.patterns if p.pattern_type == pattern_type]


class DTWPatternMatcher:
    """Match price patterns using Dynamic Time Warping."""

    def __init__(
        self,
        window_sizes: list[int] | None = None,
        similarity_threshold: float = 0.7,
    ) -> None:
        """Initialize matcher.

        Args:
            window_sizes: Window sizes to search for patterns
            similarity_threshold: Minimum similarity to report match
        """
        self.window_sizes = window_sizes or [15, 20, 30, 40, 50]
        self.similarity_threshold = similarity_threshold
        self.library = PatternLibrary()

    def dtw_distance(
        self,
        seq1: NDArray[Any],
        seq2: NDArray[Any],
        window: int | None = None,
    ) -> float:
        """Compute DTW distance between two sequences.

        Args:
            seq1: First sequence
            seq2: Second sequence
            window: Warping window constraint (None for no constraint)

        Returns:
            DTW distance (lower = more similar)
        """
        n, m = len(seq1), len(seq2)

        # Initialize cost matrix
        dtw = np.full((n + 1, m + 1), np.inf)
        dtw[0, 0] = 0

        # Fill DTW matrix
        for i in range(1, n + 1):
            j_start = max(1, i - window) if window else 1
            j_end = min(m + 1, i + window + 1) if window else m + 1

            for j in range(j_start, j_end):
                cost = abs(seq1[i - 1] - seq2[j - 1])
                dtw[i, j] = cost + min(
                    dtw[i - 1, j],  # insertion
                    dtw[i, j - 1],  # deletion
                    dtw[i - 1, j - 1],  # match
                )

        return float(dtw[n, m])

    def normalize_sequence(self, seq: NDArray[Any]) -> NDArray[Any]:
        """Normalize sequence to 0-1 range."""
        min_val = np.min(seq)
        max_val = np.max(seq)
        if max_val - min_val < 1e-10:
            return np.zeros_like(seq)
        return (seq - min_val) / (max_val - min_val)

    def find_patterns(
        self,
        prices: NDArray[Any],
        patterns: list[Pattern] | None = None,
    ) -> list[PatternMatch]:
        """Find patterns in price data.

        Args:
            prices: Price sequence to search
            patterns: Patterns to search for (None = use library)

        Returns:
            List of pattern matches
        """
        if patterns is None:
            patterns = self.library.patterns

        matches = []

        for window_size in self.window_sizes:
            if len(prices) < window_size:
                continue

            for pattern in patterns:
                pattern_matches = self._find_pattern_in_window(
                    prices, pattern, window_size
                )
                matches.extend(pattern_matches)

        # Remove duplicates (keep best match per region)
        matches = self._deduplicate_matches(matches)

        return matches

    def _find_pattern_in_window(
        self,
        prices: NDArray[Any],
        pattern: Pattern,
        window_size: int,
    ) -> list[PatternMatch]:
        """Find pattern using sliding window."""
        matches = []
        template = pattern.template

        # Resample template to match window size
        resampled = np.interp(
            np.linspace(0, 1, window_size),
            np.linspace(0, 1, len(template)),
            template,
        )

        stride = max(1, window_size // 4)

        for i in range(0, len(prices) - window_size + 1, stride):
            window = prices[i : i + window_size]
            normalized = self.normalize_sequence(window)

            # Compute DTW distance
            distance = self.dtw_distance(normalized, resampled, window=window_size // 4)

            # Convert distance to similarity (0-1)
            max_distance = window_size * 2  # Rough upper bound
            similarity = max(0, 1 - distance / max_distance)

            if similarity >= self.similarity_threshold:
                matches.append(
                    PatternMatch(
                        pattern_type=pattern.pattern_type,
                        similarity=similarity,
                        start_idx=i,
                        end_idx=i + window_size,
                        dtw_distance=distance,
                        description=pattern.description,
                        expected_outcome=pattern.expected_outcome,
                    )
                )

        return matches

    def _deduplicate_matches(self, matches: list[PatternMatch]) -> list[PatternMatch]:
        """Remove overlapping matches, keeping best ones."""
        if not matches:
            return []

        # Sort by similarity (descending)
        matches = sorted(matches, key=lambda m: m.similarity, reverse=True)

        deduplicated = []
        used_ranges: list[tuple[int, int]] = []

        for match in matches:
            # Check if this range overlaps with any used range
            overlaps = False
            for start, end in used_ranges:
                if not (match.end_idx <= start or match.start_idx >= end):
                    overlap_pct = (
                        min(match.end_idx, end) - max(match.start_idx, start)
                    ) / (match.end_idx - match.start_idx)
                    if overlap_pct > 0.5:
                        overlaps = True
                        break

            if not overlaps:
                deduplicated.append(match)
                used_ranges.append((match.start_idx, match.end_idx))

        return deduplicated

    def match_single_pattern(
        self,
        prices: NDArray[Any],
        pattern_type: PatternType,
    ) -> PatternMatch | None:
        """Find best match for a specific pattern type.

        Args:
            prices: Price sequence
            pattern_type: Pattern type to search for

        Returns:
            Best match or None
        """
        patterns = self.library.get_patterns_by_type(pattern_type)
        if not patterns:
            return None

        matches = self.find_patterns(prices, patterns)
        if not matches:
            return None

        return max(matches, key=lambda m: m.similarity)

    def create_custom_pattern(
        self,
        name: str,
        template_prices: NDArray[Any],
        expected_outcome: str = "neutral",
        description: str = "",
    ) -> Pattern:
        """Create a custom pattern from price data.

        Args:
            name: Pattern name
            template_prices: Example price sequence
            expected_outcome: "bullish", "bearish", or "neutral"
            description: Pattern description

        Returns:
            New Pattern object
        """
        normalized = self.normalize_sequence(template_prices)

        pattern = Pattern(
            name=name,
            pattern_type=PatternType.CUSTOM,
            template=normalized,
            expected_outcome=expected_outcome,
            description=description or f"Custom pattern: {name}",
        )

        return pattern
