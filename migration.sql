-- HotDog Database Migration
-- SQLite Schema for tracking kids, notes, and settings

-- Settings table: Stores application-wide settings like aggregation granularity
CREATE TABLE IF NOT EXISTS "settings" (
    `id` integer PRIMARY KEY AUTOINCREMENT,
    `granularity` text NOT NULL,
    `created_at` text DEFAULT 'datetime("now", "utc")'
);

-- Kids table: Stores information about each kid being tracked
CREATE TABLE IF NOT EXISTS "kids" (
    `id` integer PRIMARY KEY AUTOINCREMENT UNIQUE,
    `name` text NOT NULL,
    `created_at` text DEFAULT 'datetime("now", "utc")'
);

-- Notes table: Stores individual notes/tallies for each kid
CREATE TABLE IF NOT EXISTS "notes" (
    `id` integer PRIMARY KEY,
    `created_at` text DEFAULT 'datetime(''now'', ''utc'')' NOT NULL,
    `quantity` integer,
    `kid_id` integer NOT NULL,
    FOREIGN KEY (`kid_id`) REFERENCES `kids`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Index for efficient queries on notes by kid_id and quantity
CREATE INDEX IF NOT EXISTS `kid_id_idx` ON `notes` (`kid_id`,`quantity`);

-- Optional: Insert default settings (MONTHLY aggregation)
INSERT INTO settings (granularity) VALUES ('MONTHLY');

-- Optional: Insert sample kids (uncomment if needed for testing)
-- INSERT INTO kids (name) VALUES ('Junior');
-- INSERT INTO kids (name) VALUES ('Niklas');
-- INSERT INTO kids (name) VALUES ('Alina');
