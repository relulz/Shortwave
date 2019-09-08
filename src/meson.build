# Resources
resource_conf = configuration_data()
resource_conf.set_quoted('RESOURCEFILE', app_resources.full_path())
resource_rs = configure_file(
  input: 'static_resource.rs.in',
  output: 'static_resource.rs',
  configuration: resource_conf
)

run_command(
  'cp',
  resource_rs,
  meson.current_source_dir(),
  check: true
)


# Configuration file
conf = configuration_data()
conf.set_quoted('NAME', name)
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('PROFILE', profile)
conf.set_quoted('VCS_TAG', vcs_tag)
conf.set_quoted('LOCALEDIR', localedir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)


# Source code itself
sources = files(
  'api/client.rs',
  'api/mod.rs',
  'api/object.rs',
  'api/station.rs',
  'api/station_request.rs',
  'api/station_url.rs',

  'audio/controller/sidebar_controller.rs',
  'audio/controller/mini_controller.rs',
  'audio/controller/mod.rs',
  'audio/controller/mpris_controller.rs',
  'audio/gstreamer_backend.rs',
  'audio/mod.rs',
  'audio/playback_state.rs',
  'audio/player.rs',
  'audio/song.rs',

  'database/connection.rs',
  'database/gradio_db.rs',
  'database/library.rs',
  'database/mod.rs',
  'database/queries.rs',
  'database/schema.rs',
  'discover/pages/mod.rs',
  'discover/pages/search.rs',
  'discover/mod.rs',
  'discover/storefront.rs',
  'discover/tile_button.rs',

  'model/mod.rs',
  'model/object_wrapper.rs',
  'model/song_model.rs',

  'ui/mod.rs',
  'ui/notification.rs',
  'ui/song_listbox.rs',
  'ui/song_row.rs',
  'ui/station_dialog.rs',
  'ui/station_flowbox.rs',
  'ui/station_row.rs',
  'ui/window.rs',

  'app.rs',
  'config.rs',
  'main.rs',
  'path.rs',
  'static_resource.rs',
  'utils.rs'
)

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  depends: app_resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    localedir,
    profile
  ]
)
