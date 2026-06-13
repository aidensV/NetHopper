PRAGMA foreign_keys = ON;
-- DROP TABLE IF EXISTS groups;
-- DROP TABLE IF EXISTS hosts; 
-- DROP TABLE IF EXISTS passwords; 
-- DROP TABLE IF EXISTS tunnels; 
/* =========================
   TABLE: groups
   ========================= */
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parent_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (parent_id)
        REFERENCES groups(id)
        ON DELETE CASCADE
);

/* =========================
   TABLE: hosts
   ========================= */
CREATE TABLE IF NOT EXISTS hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 22,
    username TEXT NOT NULL,
    password_id INTEGER ,
    auth_type TEXT NOT NULL, -- password | key
    group_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (group_id)
        REFERENCES groups(id)
        ON DELETE CASCADE
);

/* =========================
   INDEX (PERFORMANCE)
   ========================= */
CREATE INDEX IF NOT EXISTS idx_groups_parent
    ON groups(parent_id);

CREATE INDEX IF NOT EXISTS idx_hosts_group
    ON hosts(group_id);



/* =========================
   TABLE: passwords
   ========================= */
CREATE TABLE IF NOT EXISTS passwords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    password TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tunnels (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    type        TEXT NOT NULL CHECK(type IN ('local', 'socks5')),
    host_id     INTEGER NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,

    -- Local Port Forwarding
    local_port  INTEGER NOT NULL,
    remote_host TEXT,             -- hanya untuk local port forwarding
    remote_port INTEGER,          -- hanya untuk local port forwarding

    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);