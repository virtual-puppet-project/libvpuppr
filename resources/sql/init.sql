begin;

/*
App metadata. Basically, any data that is not directly modifiable by the user.
*/
create table Metadata (
    last_startup timestamp,
    last_shutdown timestamp,
    last_used_runner_data uuid,

    base_options uuid
);

/*
Data for a Runner, which is a grouping of resources needed for vpuppr to work.
The id is used for querying all the other tables for data.
*/
create table RunnerData (
    id uuid primary key not null,
    name text,
    runner_path text,
    gui_path text,
    model_path text,
    preview_path text,
    is_favorite boolean,
    last_used timestamp
);

/*
General app options. Applicable for all puppets.
*/
create table General (
    parent uuid unique not null,

    window_size map,
    window_screen integer
);

/*
Custom options. This is the only free-floating table and is used for storing
data from 3rd-party plugins.
*/
create table Custom (
    parent uuid unique not null,
);

create table IFacialMocap (
    parent uuid unique not null,

    address inet,
    port integer
);

create table VTubeStudio (
    parent uuid unique not null,

    address inet,
    port integer
);

create table MeowFace (
    parent uuid unique not null,

    address inet,
    port integer
);

create table MediaPipe (
    parent uuid unique not null,

    camera_resolution map
);

create table Puppet3d (
    parent uuid unique not null,

    head_bone text
);

create table IkTargetTransforms (
    parent uuid unique not null,

    head map,
    left_hand map,
    right_hand map,
    hips map,
    left_foot map,
    right_foot map
);

create table GlbPuppet (
    parent uuid unique not null
);

create table VrmPuppet (
    parent uuid unique not null,

    blink_threshold float,
    link_eye_blinks float,
    use_raw_eye_rotation boolean
);

create table Puppet2d (
    parent uuid unique not null
);

create table PngPuppet (
    parent uuid unique not null
);

commit;
