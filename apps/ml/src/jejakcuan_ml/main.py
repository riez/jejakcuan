"""JejakCuan ML Service - FastAPI application."""

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
    # Startup: Load models (non-blocking if not available)
    try:
        lstm_predictor.load(settings.lstm_model_path)
    except Exception:
        pass  # Model will be loaded on first request or via /predict/load

    try:
        sentiment_analyzer.load(settings.sentiment_model_path)
    except Exception:
        pass  # Model will be loaded on demand

    yield
    # Shutdown: Cleanup if needed


app = FastAPI(
    title="JejakCuan ML Service",
    description="Machine learning service for Indonesian stock prediction and sentiment analysis",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure properly in production
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
    return {"message": "JejakCuan ML Service v1.0.0"}


@app.get("/health")
async def health_check() -> dict[str, str]:
    """Health check endpoint."""
    return {"status": "healthy", "service": "ml"}
