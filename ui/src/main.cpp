#include "EngineProxy.h"
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

int main(int argc, char *argv[]) {
  QGuiApplication app(argc, argv);
  QQmlApplicationEngine engine;

  EngineProxy proxy;
  engine.rootContext()->setContextProperty("engineProxy", &proxy);

  engine.loadFromModule("Lancea", "Main");
  if (engine.rootObjects().isEmpty())
    return 1;
  return app.exec();
}
