# Configuration file
conf = configuration_data()
conf.set_quoted('NAME', name)
conf.set_quoted('PKGNAME', meson.project_name())
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('PROFILE', profile)
conf.set_quoted('VCS_TAG', vcs_tag)
conf.set_quoted('LOCALEDIR', localedir)
conf.set_quoted('PKGDATADIR', pkgdatadir)

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
  'api/error.rs',
  'api/favicon_downloader.rs',
  'api/mod.rs',
  'api/object.rs',
  'api/station.rs',
  'api/station_metadata.rs',
  'api/station_request.rs',
  'api/station_url.rs',
  'api/stats.rs',

  'audio/backend/gstreamer_backend.rs',
  'audio/backend/mod.rs',
  'audio/backend/song_backend.rs',
  'audio/controller/gcast_controller.rs',
  'audio/controller/inhibit_controller.rs',
  'audio/controller/sidebar_controller.rs',
  'audio/controller/toolbar_controller.rs',
  'audio/controller/mod.rs',
  'audio/controller/mpris_controller.rs',
  'audio/controller/mini_controller.rs',
  'audio/gcast_discoverer.rs',
  'audio/mod.rs',
  'audio/player.rs',
  'audio/song.rs',

  'database/connection.rs',
  'database/library.rs',
  'database/mod.rs',
  'database/queries.rs',
  'database/schema.rs',

  'settings/key.rs',
  'settings/mod.rs',
  'settings/settings_window.rs',
  'settings/settings_manager.rs',

  'model/mod.rs',
  'model/station_model.rs',
  'model/station_sorter.rs',

  'ui/pages/discover_page.rs',
  'ui/pages/library_page.rs',
  'ui/pages/mod.rs',
  'ui/pages/search_page.rs',
  'ui/about_dialog.rs',
  'ui/featured_carousel.rs',
  'ui/mod.rs',
  'ui/notification.rs',
  'ui/song_listbox.rs',
  'ui/song_row.rs',
  'ui/station_favicon.rs',
  'ui/station_dialog.rs',
  'ui/station_flowbox.rs',
  'ui/station_row.rs',
  'ui/streaming_dialog.rs',
  'ui/window.rs',

  'app.rs',
  'config.rs',
  'main.rs',
  'path.rs'
)

outputname = meson.project_name()
if host_machine.system() == 'windows'
  outputname = '@0@.exe'.format(outputname)
endif

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: outputname,
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    localedir,
    profile
  ]
)
