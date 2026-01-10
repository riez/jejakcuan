"""Price prediction endpoints."""

import uuid
from datetime import datetime
from typing import Any

import numpy as np
from fastapi import APIRouter, BackgroundTasks, HTTPException

from ..models import lstm_predictor
from ..schemas import (
    BatchPredictionRequest,
    BatchPredictionResponse,
    ModelStatus,
    PredictionWithFeatures,
    PricePredictionRequest,
    PricePredictionResponse,
    TrainingJobStatus,
    TrainingRequest,
    TrainingResponse,
    TrainingStatus,
)

router = APIRouter(prefix="/predict", tags=["prediction"])

# In-memory training job tracker (use Redis in production)
_training_jobs: dict[str, TrainingJobStatus] = {}


@router.post("/price", response_model=PricePredictionResponse)
async def predict_price(request: PricePredictionRequest) -> PricePredictionResponse:
    """Predict price direction for a stock.

    Returns UP, DOWN, or SIDEWAYS prediction with confidence score.
    """
    if not lstm_predictor.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded. Call /predict/load first.")

    return lstm_predictor.predict(request.symbol, request.horizon_days)


@router.post("/price/batch", response_model=BatchPredictionResponse)
async def predict_price_batch(request: BatchPredictionRequest) -> BatchPredictionResponse:
    """Batch predict price direction for multiple stocks."""
    if not lstm_predictor.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    predictions = []
    for symbol in request.symbols:
        pred = lstm_predictor.predict(symbol, request.horizon_days)
        predictions.append(pred)

    return BatchPredictionResponse(
        predictions=predictions,
        timestamp=datetime.utcnow(),
        model_version=lstm_predictor.version,
    )


@router.post("/price/features", response_model=PricePredictionResponse)
async def predict_from_features(request: PredictionWithFeatures) -> PricePredictionResponse:
    """Predict from pre-computed features.

    Useful when features are computed externally (e.g., by Rust backend).
    """
    if not lstm_predictor.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    try:
        features = np.array(request.features)
        return lstm_predictor.predict_from_features(
            features,
            symbol=request.symbol,
            horizon_days=request.horizon_days,
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/status", response_model=ModelStatus)
async def get_model_status() -> ModelStatus:
    """Get LSTM model status and metrics."""
    return ModelStatus(
        name="LSTM Price Predictor",
        loaded=lstm_predictor.loaded,
        version=lstm_predictor.version if lstm_predictor.loaded else None,
        last_trained=None,  # TODO: Load from metadata
        metrics=None,  # TODO: Load validation metrics
    )


@router.post("/load")
async def load_model(model_path: str | None = None) -> dict[str, Any]:
    """Load or reload the LSTM model.

    Args:
        model_path: Optional path to model. Uses default if not specified.
    """
    from ..config import settings

    path = model_path or settings.lstm_model_path

    try:
        success = lstm_predictor.load(path)
        if success:
            return {
                "status": "loaded",
                "version": lstm_predictor.version,
                "path": path,
            }
        else:
            raise HTTPException(status_code=500, detail="Failed to load model")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/train", response_model=TrainingResponse)
async def trigger_training(
    request: TrainingRequest,
    background_tasks: BackgroundTasks,
) -> TrainingResponse:
    """Trigger model training (async).

    Training runs in background. Use /train/{job_id}/status to check progress.
    """
    job_id = str(uuid.uuid4())[:8]

    # Create job status
    job = TrainingJobStatus(
        job_id=job_id,
        status=TrainingStatus.PENDING,
        progress=0.0,
        started_at=datetime.utcnow(),
    )
    _training_jobs[job_id] = job

    # Queue background task
    background_tasks.add_task(
        _run_training,
        job_id=job_id,
        symbols=request.symbols,
        epochs=request.epochs,
    )

    return TrainingResponse(
        job_id=job_id,
        status=TrainingStatus.PENDING,
        message="Training job queued",
        started_at=job.started_at,
    )


@router.get("/train/{job_id}/status", response_model=TrainingJobStatus)
async def get_training_status(job_id: str) -> TrainingJobStatus:
    """Get status of a training job."""
    if job_id not in _training_jobs:
        raise HTTPException(status_code=404, detail="Training job not found")

    return _training_jobs[job_id]


async def _run_training(job_id: str, symbols: list[str], epochs: int) -> None:
    """Background training task."""
    job = _training_jobs[job_id]
    job.status = TrainingStatus.RUNNING

    try:
        # TODO: Implement actual training
        # 1. Fetch price data for symbols
        # 2. Extract features
        # 3. Train model
        # 4. Save model

        # For now, just simulate
        import asyncio

        for i in range(10):
            await asyncio.sleep(0.5)
            job.progress = (i + 1) / 10

        job.status = TrainingStatus.COMPLETED
        job.completed_at = datetime.utcnow()
        job.metrics = {"accuracy": 0.65, "samples": 1000}

    except Exception as e:
        job.status = TrainingStatus.FAILED
        job.error = str(e)
        job.completed_at = datetime.utcnow()
