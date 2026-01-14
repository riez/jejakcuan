"""Telegram channel monitoring for Indonesian stock groups."""

import asyncio
from dataclasses import dataclass
from datetime import datetime
from typing import Callable

from .parser import TelegramMessageParser


@dataclass
class TelegramMessage:
    """Parsed Telegram message."""

    message_id: int
    channel_id: str
    channel_name: str
    text: str
    timestamp: datetime
    sender_name: str | None = None
    reply_to: int | None = None
    views: int = 0
    forwards: int = 0


@dataclass
class MonitorConfig:
    """Telegram monitor configuration."""

    api_id: str
    api_hash: str
    phone_number: str | None = None
    session_name: str = "jejakcuan_monitor"
    channels: list[str] | None = None  # Channel usernames to monitor


class TelegramMonitor:
    """Monitor Telegram channels for stock-related messages.

    Note: Requires telethon library and Telegram API credentials.
    For production use, ensure compliance with Telegram ToS.
    """

    # Default Indonesian stock discussion channels
    DEFAULT_CHANNELS = [
        "stockbitfeed",
        "indonesiastockwatch",
        "sahamology",
    ]

    def __init__(self, config: MonitorConfig) -> None:
        """Initialize monitor.

        Args:
            config: Telegram API configuration
        """
        self.config = config
        self.parser = TelegramMessageParser()
        self._client = None
        self._running = False
        self._message_handlers: list[Callable[[TelegramMessage], None]] = []

    @property
    def channels(self) -> list[str]:
        """Get list of channels to monitor."""
        return self.config.channels or self.DEFAULT_CHANNELS

    def add_handler(self, handler: Callable[[TelegramMessage], None]) -> None:
        """Add message handler callback.

        Args:
            handler: Function called with each new message
        """
        self._message_handlers.append(handler)

    async def connect(self) -> bool:
        """Connect to Telegram.

        Returns:
            True if connected successfully
        """
        try:
            # Defer telethon import to avoid requiring it at module load
            from telethon import TelegramClient

            self._client = TelegramClient(
                self.config.session_name,
                self.config.api_id,
                self.config.api_hash,
            )

            await self._client.start(phone=self.config.phone_number)
            return await self._client.is_user_authorized()

        except ImportError:
            print("telethon not installed. Install with: pip install telethon")
            return False
        except Exception as e:
            print(f"Failed to connect to Telegram: {e}")
            return False

    async def disconnect(self) -> None:
        """Disconnect from Telegram."""
        if self._client:
            await self._client.disconnect()
            self._client = None

    async def get_channel_history(
        self,
        channel: str,
        limit: int = 100,
        offset_date: datetime | None = None,
    ) -> list[TelegramMessage]:
        """Fetch recent messages from a channel.

        Args:
            channel: Channel username or ID
            limit: Maximum messages to fetch
            offset_date: Only fetch messages before this date

        Returns:
            List of parsed messages
        """
        if not self._client:
            return []

        messages: list[TelegramMessage] = []

        try:
            entity = await self._client.get_entity(channel)

            async for msg in self._client.iter_messages(
                entity,
                limit=limit,
                offset_date=offset_date,
            ):
                if msg.text:
                    # Parse and filter stock-related messages
                    parsed = TelegramMessage(
                        message_id=msg.id,
                        channel_id=str(entity.id),
                        channel_name=getattr(entity, "username", channel),
                        text=msg.text,
                        timestamp=msg.date,
                        sender_name=getattr(msg.sender, "username", None),
                        reply_to=getattr(msg.reply_to, "reply_to_msg_id", None),
                        views=msg.views or 0,
                        forwards=msg.forwards or 0,
                    )

                    # Only include if stock-related
                    if self.parser.is_stock_related(msg.text):
                        messages.append(parsed)

        except Exception as e:
            print(f"Error fetching messages from {channel}: {e}")

        return messages

    async def start_monitoring(self) -> None:
        """Start real-time message monitoring."""
        if not self._client:
            return

        self._running = True

        try:
            from telethon import events

            @self._client.on(events.NewMessage(chats=self.channels))
            async def handler(event):
                if not event.text:
                    return

                # Check if stock-related
                if not self.parser.is_stock_related(event.text):
                    return

                msg = TelegramMessage(
                    message_id=event.id,
                    channel_id=str(event.chat_id),
                    channel_name=getattr(event.chat, "username", ""),
                    text=event.text,
                    timestamp=event.date,
                    sender_name=getattr(event.sender, "username", None),
                )

                # Call all handlers
                for h in self._message_handlers:
                    try:
                        h(msg)
                    except Exception as e:
                        print(f"Handler error: {e}")

            # Keep running until stopped
            while self._running:
                await asyncio.sleep(1)

        except ImportError:
            print("telethon not installed")
        except Exception as e:
            print(f"Monitoring error: {e}")
        finally:
            self._running = False

    async def stop_monitoring(self) -> None:
        """Stop real-time monitoring."""
        self._running = False

    async def fetch_all_channels(
        self,
        limit_per_channel: int = 100,
    ) -> dict[str, list[TelegramMessage]]:
        """Fetch messages from all monitored channels.

        Args:
            limit_per_channel: Max messages per channel

        Returns:
            Dict mapping channel name to messages
        """
        results: dict[str, list[TelegramMessage]] = {}

        for channel in self.channels:
            messages = await self.get_channel_history(channel, limit_per_channel)
            results[channel] = messages

        return results
