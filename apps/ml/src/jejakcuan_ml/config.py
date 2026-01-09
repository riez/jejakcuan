"""Configuration settings for ML service."""

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application settings."""

    model_config = SettingsConfigDict(env_prefix="JEJAKCUAN_ML_")

    app_name: str = "JejakCuan ML Service"
    debug: bool = False

    # Model paths
    lstm_model_path: str = "models/lstm"
    sentiment_model_path: str = "models/indobert"

    # API settings
    api_prefix: str = "/api/v1"


settings = Settings()
