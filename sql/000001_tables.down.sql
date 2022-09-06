
--drops tables
BEGIN;

DROP TABLE IF EXISTS "eth_prices";
DROP TABLE IF EXISTS "btc_prices";
DROP TABLE IF EXISTS "crypto_prices";
DROP TABLE IF EXISTS "currency_info";

COMMIT;