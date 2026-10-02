CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL,
    password_hash TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    -- Copied into every JWT; bumping it revokes all the user's tokens.
    token_version INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX users_username_key ON users (lower(username));

CREATE TABLE invitations (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX invitations_user_id_idx ON invitations (user_id);

CREATE TABLE lists (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX lists_owner_id_idx ON lists (owner_id);

CREATE TABLE participants (
    id UUID PRIMARY KEY,
    list_id UUID NOT NULL REFERENCES lists (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    turn_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (list_id, turn_order)
);

CREATE TABLE films (
    id UUID PRIMARY KEY,
    list_id UUID NOT NULL REFERENCES lists (id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    -- clock_timestamp(), not now(): films added in one transaction keep their order.
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE INDEX films_list_id_idx ON films (list_id);

CREATE TABLE selections (
    id UUID PRIMARY KEY,
    list_id UUID NOT NULL REFERENCES lists (id) ON DELETE CASCADE,
    starter_id UUID REFERENCES participants (id) ON DELETE SET NULL,
    starter_name TEXT NOT NULL,
    film_count INTEGER NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX selections_list_id_idx ON selections (list_id, started_at);

-- A list runs at most one selection at a time.
CREATE UNIQUE INDEX selections_active_key ON selections (list_id) WHERE completed_at IS NULL;

CREATE TABLE eliminations (
    id UUID PRIMARY KEY,
    selection_id UUID NOT NULL REFERENCES selections (id) ON DELETE CASCADE,
    film_id UUID REFERENCES films (id) ON DELETE SET NULL,
    film_title TEXT NOT NULL,
    participant_id UUID REFERENCES participants (id) ON DELETE SET NULL,
    participant_name TEXT NOT NULL,
    place INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (selection_id, place),
    UNIQUE (selection_id, film_id)
);

CREATE INDEX eliminations_film_id_idx ON eliminations (film_id);

CREATE TABLE verdicts (
    id UUID PRIMARY KEY,
    selection_id UUID NOT NULL REFERENCES selections (id) ON DELETE CASCADE,
    film_id UUID REFERENCES films (id) ON DELETE SET NULL,
    film_title TEXT NOT NULL,
    place INTEGER NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('watched', 'abandoned')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (selection_id, place)
);
