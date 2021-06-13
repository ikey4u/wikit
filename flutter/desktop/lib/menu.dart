import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:nativeshell/nativeshell.dart';
import 'package:nativeshell/accelerators.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';

class MenuPage extends StatefulWidget {
  const MenuPage();

  @override
  State<MenuPage> createState() => _MenuPageState();
}

class _MenuPageState extends State<MenuPage> {

  bool check1 = true;
  bool check2 = false;
  int radioValue = 0;
  late Menu menu;

  @override
  void initState() {
    super.initState();
    menu = Menu(_buildMenu);
  }

  @override
  Widget build(BuildContext context) {
    var dictmenu = Container(
      decoration: BoxDecoration(
          border: Border.all(color: Colors.white, width: 1),
          color: Colors.white.withOpacity(0.15)
      ),
      child: Column(
        children: [
          MenuBar(
            menu: menu,
            itemBuilder: _buildMenuBarItem,
          ),
          if (Platform.isMacOS)
            Padding(
              padding: const EdgeInsets.symmetric(
                  horizontal: 10, vertical: 6),
              child: Text(
                  'Look up! On macOS the MenuBar is at the top of screen.'),
            )
        ],
      ),
    );

    var dictmening = GestureDetector(
      onSecondaryTapDown: _showContextMenu,
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: Colors.blue.shade300),
          color: Colors.blue.shade100,
        ),
        child: Padding(
          padding: const EdgeInsets.all(38.0),
          child: Text('Right-click here for context menu'),
        ),
      ),
    );

    var dictlayout = Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        dictmenu,
        SizedBox(
          height: 20,
        ),
        dictmening,
      ],
    );

    return dictlayout;
  }

  int _counter = 0;

  void _showContextMenu(TapDownDetails e) async {
    final menu = Menu(_buildContextMenu);

    // Menu can be updated while visible
    final timer = Timer.periodic(Duration(milliseconds: 500), (timer) {
      ++_counter;
      menu.update();
    });

    await Window.of(context).showPopupMenu(menu, e.globalPosition);

    timer.cancel();
  }

  List<MenuItem> _buildContextMenu() => [
        MenuItem(title: 'Context menu Item', action: () {}),
        MenuItem(title: 'Menu Update Counter $_counter', action: null),
        MenuItem.separator(),
        ..._buildCheckAndRadioItems(),
        MenuItem.separator(),
        MenuItem.children(title: 'Submenu', children: [
          MenuItem(title: 'Submenu Item 1', action: () {}),
          MenuItem(title: 'Submenu Item 2', action: () {}),
        ]),
      ];

  Widget _buildMenuBarItem(BuildContext context, Widget child, MenuItemState itemState) {
    Color background;
    Color foreground;
    switch (itemState) {
      case MenuItemState.regular:
        background = Colors.transparent;
        foreground = Colors.grey.shade800;
        break;
      case MenuItemState.hovered:
        background = Colors.purple.withOpacity(0.2);
        foreground = Colors.grey.shade800;
        break;
      case MenuItemState.selected:
        background = Colors.purple.withOpacity(0.8);
        foreground = Colors.white;
        break;
      case MenuItemState.disabled:
        background = Colors.transparent;
        foreground = Colors.grey.shade800.withOpacity(0.5);
        break;
    }
    return Container(
      padding: EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      color: background,
      child: DefaultTextStyle.merge(
        style: TextStyle(color: foreground),
        child: child,
      ),
    );
  }


  // This will be the default "fallback" app menu used for any window that doesn't
  // have other menu
  List<MenuItem> _buildMenu() => [
        if (Platform.isMacOS)
          MenuItem.children(title: 'App', children: [
            MenuItem.withRole(role: MenuItemRole.hide),
            MenuItem.withRole(role: MenuItemRole.hideOtherApplications),
            MenuItem.withRole(role: MenuItemRole.showAll),
            MenuItem.separator(),
            MenuItem.withRole(role: MenuItemRole.quitApplication),
          ]),
        MenuItem.children(title: '&File', children: [
          MenuItem(title: 'New', accelerator: cmdOrCtrl + 'n', action: () {}),
          MenuItem(title: 'Open', accelerator: cmdOrCtrl + 'o', action: () {}),
          MenuItem.separator(),
          MenuItem(title: 'Save', accelerator: cmdOrCtrl + 's', action: null),
          MenuItem(title: 'Save As', action: null),
          MenuItem.separator(),
          MenuItem(title: 'Close', action: () {}),
        ]),
        MenuItem.children(title: '&Edit', children: [
          MenuItem(title: 'Cut', accelerator: cmdOrCtrl + 'x', action: () {}),
          MenuItem(title: 'Copy', accelerator: cmdOrCtrl + 'c', action: () {}),
          MenuItem(title: 'Paste', accelerator: cmdOrCtrl + 'v', action: () {}),
          MenuItem.separator(),
          MenuItem(title: 'Find', accelerator: cmdOrCtrl + 'f', action: () {}),
          MenuItem(title: 'Replace', action: () {}),
        ]),
        MenuItem.children(title: '&Tools', children: [
          ..._buildCheckAndRadioItems(),
          MenuItem.separator(),
          MenuItem.children(title: 'Submenu', children: [
            MenuItem(title: 'More of the same, I guess?', action: null),
            MenuItem.separator(),
            ..._buildCheckAndRadioItems(),
          ]),
        ]),
        if (Platform.isMacOS)
          MenuItem.children(title: 'Window', role: MenuRole.window, children: [
            MenuItem.withRole(role: MenuItemRole.minimizeWindow),
            MenuItem.withRole(role: MenuItemRole.zoomWindow),
          ]),
        MenuItem.children(title: '&Help', children: [
          MenuItem(title: 'About', action: () {}),
        ]),
      ];

  List<MenuItem> _buildCheckAndRadioItems() => [
        MenuItem(
            title: 'Checkable Item 1',
            checkStatus: check1 ? CheckStatus.checkOn : CheckStatus.checkOff,
            action: () {
              check1 = !check1;
              menu.update();
            }),
        MenuItem(
            title: 'Checkable Item 2',
            checkStatus: check2 ? CheckStatus.checkOn : CheckStatus.checkOff,
            action: () {
              check2 = !check2;
              menu.update();
            }),
        MenuItem.separator(),
        MenuItem(
            title: 'Radio Item 1',
            checkStatus:
                radioValue == 0 ? CheckStatus.radioOn : CheckStatus.radioOff,
            action: () {
              radioValue = 0;
              menu.update();
            }),
        MenuItem(
            title: 'Radio Item 2',
            checkStatus:
                radioValue == 1 ? CheckStatus.radioOn : CheckStatus.radioOff,
            action: () {
              radioValue = 1;
              menu.update();
            }),
        MenuItem(
            title: 'Radio Item 3',
            checkStatus:
                radioValue == 2 ? CheckStatus.radioOn : CheckStatus.radioOff,
            action: () {
              radioValue = 2;
              menu.update();
            }),
      ];
}
