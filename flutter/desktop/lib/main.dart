import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:nativeshell/nativeshell.dart';
import 'package:nativeshell/accelerators.dart';

import './menu.dart';

void main() async {
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Wikit',
      home: DefaultTextStyle(
        style: TextStyle(
          color: Colors.black,
          fontSize: 24,
        ),
        child: Container(
          color: Colors.white,
          child: WindowWidget(
            onCreateState: (initData) {
              return MainWindowState();
            },
          ),
        ),
      ),
    );
  }
}

class MainWindowState extends WindowState {
  @override
  Widget build(BuildContext context) {
    return MenuPage();
  }
}
