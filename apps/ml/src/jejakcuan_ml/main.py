"""FastAPI application for JejakCuan ML service."""

from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .config import settings
from .models import lstm_predictor, sentiment_analyzer
from .routers import predict_router, sentiment_router


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Application lifespan handler."""
    # Startup: Load models
    lstm_predictor.load(settings.lstm_model_path)
    sentiment_analyzer.load(settings.sentiment_model_path)
    yield
    # Shutdown: Cleanup if needed


app = FastAPI(
    title=settings.app_name,
    version="0.1.0",
    description="ML service for stock prediction and sentiment analysis",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(predict_router, prefix=settings.api_prefix)
app.include_router(sentiment_router, prefix=settings.api_prefix)


@app.get("/")
async def root() -> dict[str, str]:
    """Root endpoint."""
    return {"message": "JejakCuan ML Service v0.1.0"}


@app.get("/health")
async def health() -> dict[str, str]:
    """Health check endpoint."""
    return {"status": "OK"}
