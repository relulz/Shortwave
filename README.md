# Shortwave
Find and listen to internet radio stations

![alt text](https://gitlab.gnome.org/World/Shortwave/raw/master/data/icons/hicolor/scalable/apps/de.haeckerfelix.Shortwave.svg "Logo")

___
**Shortwave is still in an early stage of development. It will be the successor of Gradio.**
As long as Shortwave is still in development, you can still download Gradio from [here](https://flathub.org/apps/details/de.haeckerfelix.gradio).
___

## Getting in Touch
If you have any questions regarding the use or development of Shortwave,
want to discuss design or simply hang out, please join us on our [#shortwave:matrix.org](https://matrix.to/#/#shortwave:matrix.org) channel.

## FAQ
- **Why its called 'Shortwave'?**

    Shortwave signals have a very long range because of their very good reflection properties. 
Due to their long range, they can be received almost anywhere in the world. 
The same applies to Internet radio stations, which can also be received almost anywhere in the world.
That's why we decided to call the project 'Shortwave', because internet radio stations and shortwave radio stations share many characteristics.

    If you want to know more about the naming process, you should read this [blog post](https://blogs.gnome.org/tbernard/2019/04/26/naming-your-app/)

- **Why I cannot edit stations anymore?**

    The edit feature is disabled because of vandalism. I cannot change this. [More information here](http://www.radio-browser.info/gui/#/) and [here](https://github.com/segler-alex/radiobrowser-api/issues/39)

- **Will Shortwave compatible with the Librem 5?**

    Yes! We use the awesome [libhandy](https://source.puri.sm/Librem5/libhandy) library to make the interface adaptive.

- **Which database does Shortwave use?**

    [radio-browser.info](http://www.radio-browser.info/gui/#/). It's a community database. Everybody can add/edit information.

## Development builds

#### Flatpak nightly builds
You can install nightly builds of Shortwave by using this command:

`flatpak install https://haeckerfelix.de/~repo/shortwave.flatpakref`

More information are [here available](https://haeckerfelix.de/~repo/shortwave)! The repository is automatically updated every day. You can update to the latest nightly by running `flatpak update`!

If you haven't installed Flatpak yet, you can download it from [here](https://flatpak.org/setup/).

## Building
Shortwave requires Rust nightly features like async/await, so you have to use Rust 1.39+. 
It is expected that async/await will be included in Rust 1.39 which gets released in November. 
After that you can compile Shortwave again with a stable Rust version.

### Building with Flatpak + GNOME Builder
Shortwave can be built and run with [GNOME Builder](https://wiki.gnome.org/Apps/Builder) >= 3.28.
Just clone the repo and hit the run button!

You can get Builder from [here](https://wiki.gnome.org/Apps/Builder/Downloads), and the Rust Nightly Flatpak SDK (if necessary) from [here](https://haeckerfelix.de/~repo/)

### Building it manually
1. `git clone https://gitlab.gnome.org/World/Shortwave.git`
2. `cd Shortwave`
3. `meson --prefix=/usr build`
4. `ninja -C build`
5. `sudo ninja -C build install`

You need following dependencies to build Shortwave:
- Rust 1.39 or later
- GTK 3.24 or later
- Gstreamer 1.12 or later
- [libhandy](https://source.puri.sm/Librem5/libhandy)
- [Meson Buildsystem](https://mesonbuild.com/)

If you need help to build Shortwave, please don't hesitate to ask [here](https://matrix.to/#/#shortwave:matrix.org)!

## To-Do
This is what still needs to be done for the first release of Shortwave. There's probably much more.

- [ ] Make application translatable
- [ ] Implement disover page
- [ ] Implement UI selection mode (https://developer.gnome.org/hig/stable/selection-mode.html.en)
- [ ] Implement settings dialog
- [x] Show station icons instead of placeholder icon
- [ ] Add ability to vote for stations
- [ ] Improve stream recorder (Add several record modes, automatically ignore songs that are shorter than 30 seconds, ...)
- [ ] Implement GNOME Search Provider (https://gitlab.gnome.org/World/Rust/search-provider)
- [x] Rewrite library backend using diesel crate / using a database instead of a plain json file
- [x] radio-browser.info API: Implement async backend for discover features
- ...

## Code of Conduct
Click [here](https://gitlab.gnome.org/World/Shortwave/blob/master/CODE_OF_CONDUCT.md) to learn more about our Code of Conduct.
