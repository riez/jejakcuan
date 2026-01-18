"""CLI entry point for scrapers.

Usage:
    python -m jejakcuan_ml.scrapers.cli [command] [options]

Commands:
    all           Run all scrapers
    sync-stocks   Sync stock list from IDX API to database
    eipo          Scrape e-IPO listings
    idx           Scrape IDX fundamental data
    broker        Scrape broker flow data
    price         Scrape price history

Options:
    --symbols SYMBOL [SYMBOL ...]  Specific symbols to scrape
    --days N                       Number of days of history (default: 365)
    --sharia-only                  Only sync sharia-compliant stocks (sync-stocks only)
    --help                         Show this help message

Examples:
    python -m jejakcuan_ml.scrapers.cli sync-stocks           # Sync ALL IDX stocks
    python -m jejakcuan_ml.scrapers.cli sync-stocks --sharia-only  # Sync only ISSI stocks
    python -m jejakcuan_ml.scrapers.cli all
    python -m jejakcuan_ml.scrapers.cli price --days 60
    python -m jejakcuan_ml.scrapers.cli idx BBCA BBRI TLKM
    python -m jejakcuan_ml.scrapers.cli broker --days 30 --symbols ASII UNTR
"""

import argparse
import asyncio
import sys

from loguru import logger

from .broker_flow import BrokerFlowScraper
from .eipo import EIPOScraper
from .idx import IDXScraper
from .price_history import PriceHistoryScraper


def setup_logging(verbose: bool = False) -> None:
    """Set up logging configuration.

    Args:
        verbose: Enable verbose logging
    """
    logger.remove()
    level = "DEBUG" if verbose else "INFO"
    log_fmt = "<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{message}</cyan>"
    logger.add(sys.stderr, level=level, format=log_fmt)


async def run_scraper(name: str, symbols: list[str] | None, days: int) -> int:
    """Run a specific scraper.

    Args:
        name: Scraper name
        symbols: Symbols to scrape
        days: Number of days

    Returns:
        Number of records scraped
    """
    from .base import BaseScraper

    scrapers: dict[str, type[BaseScraper]] = {
        "eipo": EIPOScraper,
        "idx": IDXScraper,
        "broker": BrokerFlowScraper,
        "price": PriceHistoryScraper,
    }

    if name not in scrapers:
        logger.error(f"Unknown scraper: {name}")
        return 0

    scraper_cls = scrapers[name]
    if name == "eipo":
        scraper = scraper_cls()
    elif name == "idx":
        scraper = IDXScraper(symbols=symbols)
    elif name == "broker":
        scraper = BrokerFlowScraper(symbols=symbols, days=days)
    else:  # price
        scraper = PriceHistoryScraper(symbols=symbols, days=days)

    return await scraper.run()


async def sync_stocks(sharia_only: bool = False) -> int:
    """Sync stock list from IDX API to database."""
    scraper = IDXScraper()
    return await scraper.sync_stocks_to_db(sharia_only=sharia_only)


async def run_all_scrapers(symbols: list[str] | None, days: int) -> int:
    """Run all scrapers in sequence."""
    total = 0

    # Run in order: e-IPO first to get new stocks, then price, then fundamentals, then broker
    scraper_order = ["eipo", "price", "idx", "broker"]

    for name in scraper_order:
        try:
            logger.info(f"\n{'=' * 60}")
            logger.info(f"Running {name.upper()} scraper")
            logger.info(f"{'=' * 60}\n")
            count = await run_scraper(name, symbols, days)
            total += count
        except Exception as e:
            logger.error(f"Scraper {name} failed: {e}")

    return total


def main() -> None:
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="JejakCuan Stock Data Scrapers",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s all                    Run all scrapers
  %(prog)s price --days 60        Get 60 days of price history
  %(prog)s idx BBCA BBRI          Get fundamental data for specific stocks
  %(prog)s broker --days 30       Get 30 days of broker flow data
        """,
    )

    parser.add_argument(
        "command",
        choices=["all", "sync-stocks", "eipo", "idx", "broker", "price"],
        help="Command to run",
    )

    parser.add_argument(
        "symbols",
        nargs="*",
        default=None,
        help="Stock symbols to scrape (optional)",
    )

    parser.add_argument(
        "--days",
        type=int,
        default=365,
        help="Number of days of history (default: 365)",
    )

    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose logging",
    )

    parser.add_argument(
        "--sharia-only",
        action="store_true",
        help="Only sync sharia-compliant stocks (sync-stocks command only)",
    )

    args = parser.parse_args()

    # Set up logging
    setup_logging(args.verbose)

    # Get symbols if provided
    symbols = args.symbols if args.symbols else None

    logger.info(f"JejakCuan Scraper - {args.command}")
    if symbols:
        logger.info(f"Symbols: {', '.join(symbols)}")
    logger.info(f"Days: {args.days}")
    logger.info("")

    try:
        if args.command == "sync-stocks":
            logger.info(f"Syncing stocks from IDX API (sharia_only={args.sharia_only})")
            count = asyncio.run(sync_stocks(sharia_only=args.sharia_only))
        elif args.command == "all":
            count = asyncio.run(run_all_scrapers(symbols, args.days))
        else:
            count = asyncio.run(run_scraper(args.command, symbols, args.days))

        logger.success(f"\nTotal records scraped: {count}")

    except KeyboardInterrupt:
        logger.warning("\nScraping interrupted by user")
        sys.exit(130)
    except Exception as e:
        logger.exception(f"Scraper failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
