## cuteedit
   cuteedit is a learning project to develop a editor.

### fork todos
1. goal: add plugins to Tau
   - dir_panel: project view;
1. reference
   - neovim-gtk   Plugin Manager; Project view

### build
```sh
meson --prefix=/usr/local -Dprofile=development build
ninja -C build
sudo ninja -C build install 
```

During development you can quickly test Tau with the following command:

```sh
ninja -C build run
```

You can run tests with:

```sh
ninja -C build test
```

1. log
```
RUST_LOG=debug  ./main    
