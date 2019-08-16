-- Table: public.url_list

-- DROP TABLE public.url_list;

CREATE TABLE public.url_list
(
    id integer NOT NULL DEFAULT nextval('url_list_id_seq'::regclass),
    url character varying(512) COLLATE pg_catalog."default",
    created_at timestamp without time zone,
    visited_at timestamp without time zone,
    CONSTRAINT url_list_pkey PRIMARY KEY (id)
)
WITH (
    OIDS = FALSE
)
TABLESPACE pg_default;

ALTER TABLE public.url_list
    OWNER to postgres;

-- Index: created_at_idx

-- DROP INDEX public.created_at_idx;

CREATE INDEX created_at_idx
    ON public.url_list USING btree
    (created_at)
    TABLESPACE pg_default;

-- Index: id_idx

-- DROP INDEX public.id_idx;

CREATE UNIQUE INDEX id_idx
    ON public.url_list USING btree
    (id)
    TABLESPACE pg_default;

-- Index: url_idx

-- DROP INDEX public.url_idx;

CREATE UNIQUE INDEX url_idx
    ON public.url_list USING btree
    (url COLLATE pg_catalog."default")
    TABLESPACE pg_default;

-- Index: visited_at_idx

-- DROP INDEX public.visited_at_idx;

CREATE INDEX visited_at_idx
    ON public.url_list USING btree
    (visited_at)
    TABLESPACE pg_default;
