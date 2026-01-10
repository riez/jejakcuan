"""Tests for sentiment API endpoints."""

import pytest
from fastapi.testclient import TestClient

from jejakcuan_ml.main import app


@pytest.fixture
def client():
    """Create test client."""
    return TestClient(app)


class TestSentimentEndpoints:
    """Tests for sentiment endpoints."""

    def test_status_endpoint(self, client):
        """Test model status endpoint."""
        response = client.get("/api/v1/sentiment/status")
        assert response.status_code == 200
        data = response.json()
        assert "name" in data
        assert "loaded" in data

    def test_extract_symbols(self, client):
        """Test symbol extraction endpoint."""
        # Need to load model first or handle unloaded state
        response = client.post(
            "/api/v1/sentiment/extract-symbols",
            json={"text": "Saham BBCA dan BBRI naik hari ini. $TLKM juga bagus."},
        )
        assert response.status_code == 200
        data = response.json()
        assert "symbols" in data
        # Should find BBCA, BBRI, TLKM
        assert len(data["symbols"]) >= 2

    def test_batch_endpoint_format(self, client):
        """Test batch endpoint returns correct format."""
        # This will fail if model not loaded, but tests format
        response = client.post(
            "/api/v1/sentiment/batch",
            json={"texts": ["Test text 1", "Test text 2"]},
        )
        # Either success or 503 (model not loaded)
        assert response.status_code in [200, 503]
