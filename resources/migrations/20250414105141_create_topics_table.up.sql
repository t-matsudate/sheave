CREATE TABLE IF NOT EXISTS topics (
    path            VARCHAR(255)    PRIMARY KEY,
    client_addr     VARCHAR(39)     NOT NULL,
    published_at    TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    unpublished_at  TIMESTAMP       DEFAULT NULL
)
