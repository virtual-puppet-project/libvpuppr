begin;

create table Metadata (
    last_startup timestamp,
    last_shutdown timestamp,
    last_used_runner_data uuid,

    base_options uuid
);

create table RunnerData (
    id uuid primary key not null,
    name text,
    runner_path text,
    gui_path text,
    model_path text,
    preview_path text,
    is_favorite boolean,
    last_used timestamp,

    options uuid
);

create table GeneralOptions (
    id uuid primary key not null,

    i_facial_mocap_options uuid,
    vtube_studio_options uuid,
    meow_face_options uuid,
    media_pipe_options uuid,
);

create table IFacialMocapOptions (
    id uuid primary key not null,

    address inet,
    port integer
);

create table VTubeStudioOptions (
    id uuid primary key not null,

    address inet,
    port integer
);

create table MeowFaceOptions (
    id uuid primary key not null,

    address inet,
    port integer
);

create table MediaPipeOptions (
    id uuid primary key not null,

    camera_resolution map
);

create table Puppet3d (
    id uuid primary key not null,

    head_bone text,
    ik_target_transforms uuid
);

create table IkTargetTransforms (
    id uuid primary key not null,

    head map,
    left_hand map,
    right_hand map,
    hips map,
    left_foot map,
    right_foot map
);

create table GlbPuppet (
    id uuid primary key not null,

    puppet uuid
);

create table VrmPuppet (
    id uuid primary key not null,

    puppet uuid,

    blink_threshold float,
    link_eye_blinks float,
    use_raw_eye_rotation boolean
);

create table Puppet2d (
    id uuid primary key not null
);

create table PngPuppet (
    id uuid primary key not null
);

commit;
