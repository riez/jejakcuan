"""Price prediction endpoints."""

from fastapi import APIRouter, HTTPException

from ..models import lstm_predictor
from ..schemas import ModelStatus, PricePredictionRequest, PricePredictionResponse

router = APIRouter(prefix="/predict", tags=["prediction"])


@router.post("/price", response_model=PricePredictionResponse)
async def predict_price(request: PricePredictionRequest) -> PricePredictionResponse:
    """Predict price direction for a stock."""
    if not lstm_predictor.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    return lstm_predictor.predict(request.symbol, request.horizon_days)


@router.get("/status", response_model=ModelStatus)
async def get_model_status() -> ModelStatus:
    """Get LSTM model status."""
    return ModelStatus(
        name="LSTM Price Predictor",
        loaded=lstm_predictor.loaded,
        version=lstm_predictor.version if lstm_predictor.loaded else None,
        last_trained=None,
    )
