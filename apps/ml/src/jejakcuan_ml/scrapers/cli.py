"""CLI entry point for scrapers.

Usage:
    python -m jejakcuan_ml.scrapers.cli [scraper] [options]

Scrapers:
    all         Run all scrapers
    eipo        Scrape e-IPO listings
    idx         Scrape IDX fundamental data
    broker      Scrape broker flow data
    price       Scrape price history

Options:
    --symbols SYMBOL [SYMBOL ...]  Specific symbols to scrape
    --days N                       Number of days of history (default: 365)
    --help                         Show this help message

Examples:
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
    log_fmt = (
        "<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | " "<cyan>{message}</cyan>"
    )
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


async def run_all_scrapers(symbols: list[str] | None, days: int) -> int:
    """Run all scrapers in sequence.

    Args:
        symbols: Symbols to scrape
        days: Number of days

    Returns:
        Total number of records scraped
    """
    total = 0

    # Run in order: e-IPO first to get new stocks, then price, then fundamentals, then broker
    scraper_order = ["eipo", "price", "idx", "broker"]

    for name in scraper_order:
        try:
            logger.info(f"\n{'='*60}")
            logger.info(f"Running {name.upper()} scraper")
            logger.info(f"{'='*60}\n")
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
        "scraper",
        choices=["all", "eipo", "idx", "broker", "price"],
        help="Scraper to run",
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

    args = parser.parse_args()

    # Set up logging
    setup_logging(args.verbose)

    # Get symbols if provided
    symbols = args.symbols if args.symbols else None

    logger.info(f"JejakCuan Scraper - {args.scraper}")
    if symbols:
        logger.info(f"Symbols: {', '.join(symbols)}")
    logger.info(f"Days: {args.days}")
    logger.info("")

    try:
        if args.scraper == "all":
            count = asyncio.run(run_all_scrapers(symbols, args.days))
        else:
            count = asyncio.run(run_scraper(args.scraper, symbols, args.days))

        logger.success(f"\nTotal records scraped: {count}")

    except KeyboardInterrupt:
        logger.warning("\nScraping interrupted by user")
        sys.exit(130)
    except Exception as e:
        logger.exception(f"Scraper failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
