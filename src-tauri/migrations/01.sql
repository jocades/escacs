pragma foreign_keys = on;

create table if not exists study (
  id integer primary key,
  name text not null,
  tree_json text not null,
  created_at text default current_timestamp,
  updated_at text default current_timestamp
) strict;
