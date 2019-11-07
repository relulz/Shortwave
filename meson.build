project('shortwave',
          version: '0.0.1',
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
pkgdatadir = join_paths(get_option('prefix'), datadir, meson.project_name())

# Change app id for development builds
if profile == 'development'
  app_id = '@0@.Devel'.format(app_id)
endif

# Setup pre-commit hook for ensuring coding style is always consistent
if profile == 'development'
    message('Setting up git pre-commit hook..')
    run_command('mkdir', '.git/hooks/')
    run_command('cp', '-f', 'build-aux/pre-commit.hook', '.git/hooks/pre-commit')
endif

i18n = import('i18n')

subdir('data')
subdir('src')
subdir('po')

# Postinstall script (Update icon cache, gsettings, ...)
meson.add_install_script('build-aux/meson/postinstall.py')
