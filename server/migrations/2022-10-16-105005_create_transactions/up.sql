-- Your SQL goes here
CREATE TABLE transactions(
    id BIGINT PRIMARY KEY,
    reservation DATE NOT NULL,
    receiver TEXT NOT NULL,
    tags TEXT[] NOT NULL,
    amount DOUBLE PRECISION NOT NULL,
    currency TEXT NOT NULL
)