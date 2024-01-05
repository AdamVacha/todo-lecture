-- Add migration script here

create table todos (
    title text primary key not null,
    msg text not null
);