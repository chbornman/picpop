"""Base schema configuration."""

from pydantic import BaseModel, ConfigDict


class CamelModel(BaseModel):
    """Base model with ORM compatibility."""

    model_config = ConfigDict(from_attributes=True)
