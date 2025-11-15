CREATE TABLE IF NOT EXISTS topics (
    id              VARCHAR(36) PRIMARY KEY,
    client_addr     VARCHAR(45) NOT NULL,
    published_at    TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
    unpublished_at  TIMESTAMP   DEFAULT NULL
)
