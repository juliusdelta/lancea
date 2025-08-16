#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

#include "AppProxy.h"
#include "ClipboardProxy.h"
#include "EngineProxy.h"

int main(int argc, char *argv[]) {
  QGuiApplication app(argc, argv);

  static ClipboardProxy s_clipboard; // static lifetime OK for app singletons
  static AppProxy s_app;

  EngineProxy proxy;
  // Register QML singletons (URI separate from your UI module for clarity)
  qmlRegisterSingletonInstance("Lancea.System", 1, 0, "Clipboard",
                               &s_clipboard);
  qmlRegisterSingletonInstance("Lancea.System", 1, 0, "App", &s_app);

  QQmlApplicationEngine engine;

  engine.rootContext()->setContextProperty("engineProxy", &proxy);

  engine.loadFromModule("Lancea", "Main");
  if (engine.rootObjects().isEmpty())
    return 1;
  return app.exec();
}
