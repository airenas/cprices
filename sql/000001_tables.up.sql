
--schema.sql
--SQL statements for a database schema cryptocurrency analysis
--Timescale Inc.
--Author: Avthar Sewrathan

BEGIN;
--Schema for currency_info table
CREATE TABLE "currency_info"(
    currency_code	VARCHAR (10),
    currency 		TEXT
);

--Schema for btc_prices table
CREATE TABLE "btc_prices"(
    time            TIMESTAMP WITH TIME ZONE NOT NULL,
    opening_price   DOUBLE PRECISION,
    highest_price   DOUBLE PRECISION,
    lowest_price    DOUBLE PRECISION,
    closing_price   DOUBLE PRECISION,
    volume_btc      DOUBLE PRECISION,
    volume_currency DOUBLE PRECISION,
    currency_code   VARCHAR (10)
);

--Schema for crypto_prices table
CREATE TABLE "crypto_prices"(
    time            TIMESTAMP WITH TIME ZONE NOT NULL,
    opening_price   DOUBLE PRECISION,
    highest_price   DOUBLE PRECISION,
    lowest_price    DOUBLE PRECISION,
    closing_price   DOUBLE PRECISION,
    volume_crypto   DOUBLE PRECISION,
    volume_btc      DOUBLE PRECISION,
    currency_code   VARCHAR (10)
);

--Schema for eth_prices table
CREATE TABLE "eth_prices"(
    time            TIMESTAMP WITH TIME ZONE NOT NULL,
    opening_price   DOUBLE PRECISION,
    highest_price   DOUBLE PRECISION,
    lowest_price    DOUBLE PRECISION,
    closing_price   DOUBLE PRECISION,
    volume_eth      DOUBLE PRECISION,
    volume_currency DOUBLE PRECISION,
    currency_code   VARCHAR (10)
);

--Timescale specific statements to create hypertables for better performance
SELECT create_hypertable('btc_prices', 'time', 'opening_price', 2);
SELECT create_hypertable('eth_prices', 'time', 'opening_price', 2);
SELECT create_hypertable('crypto_prices', 'time', 'currency_code', 2);

COMMIT;