#pragma once
#include <QGuiApplication>
#include <QObject>

class AppProxy : public QObject {
  Q_OBJECT
public:
  explicit AppProxy(QObject *parent = nullptr) : QObject(parent) {}

  Q_INVOKABLE void quit() { QGuiApplication::quit(); }

  Q_INVOKABLE bool quitOnLastWindowClosed() const {
    return QGuiApplication::quitOnLastWindowClosed();
  }
  Q_INVOKABLE void setQuitOnLastWindowClosed(bool v) {
    QGuiApplication::setQuitOnLastWindowClosed(v);
  }
};
