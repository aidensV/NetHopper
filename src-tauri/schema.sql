PRAGMA foreign_keys = ON;
-- DROP TABLE IF EXISTS groups;
-- DROP TABLE IF EXISTS hosts; 
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
    password TEXT ,
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
