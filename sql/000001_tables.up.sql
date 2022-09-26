
--schema.sql
--SQL statements for a database schema cryptocurrency analysis

BEGIN;
--Schema for crypto_prices table
CREATE TABLE "crypto_prices"(
    time            TIMESTAMP WITH TIME ZONE NOT NULL,
    opening_price   DOUBLE PRECISION,
    highest_price   DOUBLE PRECISION,
    lowest_price    DOUBLE PRECISION,
    closing_price   DOUBLE PRECISION,
    volume_crypto   DOUBLE PRECISION,
    currency_pair   VARCHAR (10),
    PRIMARY KEY (time, currency_pair)
);

--Timescale specific statements to create hypertables for better performance
SELECT create_hypertable('crypto_prices', 'time', 'currency_pair', 2);

COMMIT;
