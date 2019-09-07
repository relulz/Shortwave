gnome = import('gnome')

# Icons #
subdir('icons')

# Desktop file #
desktop_conf = configuration_data()
desktop_conf.set('icon', app_id)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: files('de.haeckerfelix.Shortwave.desktop.in'),
    output: 'de.haeckerfelix.Shortwave.desktop',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(app_id),
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(datadir, 'applications')
)

# Validate desktop file #
desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif


# Appstream metadata #
appdata_conf = configuration_data()
appdata_conf.set('appid', app_id)
appstream_file = i18n.merge_file(
  input: configure_file(
    input: files('de.haeckerfelix.Shortwave.appdata.xml.in'),
    output: 'de.haeckerfelix.Shortwave.appdata.xml',
    configuration: appdata_conf
  ),
  output: '@0@.appdata.xml'.format(app_id),
  po_dir: '../po',
  install: true,
  install_dir: join_paths(datadir, 'appdata')
)

# Validate appstream metadata #
appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util,
    args: ['validate', appstream_file]
  )
endif


# Gschema file #
install_data('de.haeckerfelix.Shortwave.gschema.xml',
  install_dir: join_paths(datadir, 'glib-2.0/schemas')
)

# Validate gschema file #
compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif


# DBUS service
service_conf = configuration_data()
service_conf.set('appid', app_id)
service_conf.set('bindir', join_paths(get_option('prefix'), get_option('bindir')))
configure_file(
  input: 'de.haeckerfelix.Shortwave.service.in',
  output: '@0@.service'.format(app_id),
  configuration: service_conf,
  install_dir: join_paths(datadir,'dbus-1', 'services')
)


# Resources
app_resources = gnome.compile_resources(
  'de.haeckerfelix.Shortwave',
  'de.haeckerfelix.Shortwave.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir()
)
