pragma foreign_keys = on;

create table if not exists user (
  id integer primary key,
  name text not null
) strict;

begin transaction;
insert into user (name) values ("Alice");
insert into user (name) values ("Bob");
commit;
