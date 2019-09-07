project('shortwave',
          version: '7.99.0',
    meson_version: '>= 0.50.0',
          license: 'GPLv3',
)

# Initialize project variables
name       = 'Shortwave'
app_id     = 'de.haeckerfelix.Shortwave'
profile    = get_option('profile')
vcs_tag    = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
localedir  = join_paths(get_option('prefix'), get_option('localedir'))
datadir    = get_option('datadir')

# Change app id for development builds
if profile == 'development'
  app_id = '@0@.Devel'.format(app_id)
endif

i18n = import('i18n')

subdir('data')
subdir('src')
subdir('po')

# Postinstall script (Update icon cache, gsettings, ...)
meson.add_install_script('build-aux/meson/postinstall.py')
