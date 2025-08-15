#include "EngineProxy.h"
#include <QDBusReply>
#include <QJsonDocument>
#include <QJsonObject>

static const char *SVC = "org.lancea.Engine1";
static const char *PATH = "/org/lancea/Engine1";
static const char *IFACE = "org.lancea.Engine1";

EngineProxy::EngineProxy(QObject *parent)
    : QObject(parent),
      m_iface(SVC, PATH, IFACE, QDBusConnection::sessionBus(), this) {

  QDBusConnection::sessionBus().connect(
      SVC, PATH, IFACE, "ResultsUpdated", this,
      SLOT(resultsUpdated(ulonglong, QString, ulonglong, QString)));

  QDBusConnection::sessionBus().connect(
      SVC, PATH, IFACE, "PreviewUpdated", this,
      SLOT(previewUpdated(ulonglong, QString, QString, QString)));

  QDBusConnection::sessionBus().connect(
      SVC, PATH, IFACE, "ProviderError", this,
      SLOT(providerError(ulonglong, QString, QString)));
}

QString EngineProxy::resolveCommand(const QString &text) {
  const QJsonObject env{{"v", "1.0"}, {"data", QJsonObject{{"text", text}}}};
  QDBusReply<QString> reply = m_iface.call(
      "ResolveCommand",
      QString::fromUtf8(QJsonDocument(env).toJson(QJsonDocument::Compact)));

  return reply.isValid() ? reply.value() : QString();
}

quint64 EngineProxy::search(const QString &text, quint64 epoch) {
  QJsonObject data{{"text", text}};
  if (epoch)
    data.insert("epoch", static_cast<qint64>(epoch)); // DBus maps to u64
  const QJsonObject env{{"v", "1.0"}, {"data", data}};

  QDBusReply<qulonglong> reply = m_iface.call(
      "Search",
      QString::fromUtf8(QJsonDocument(env).toJson(QJsonDocument::Compact)));
  return reply.isValid() ? static_cast<quint64>(reply.value()) : 0;
}

void EngineProxy::requestPreview(const QString &key, quint64 epoch) {
  const QJsonObject env{
      {"v", "1.0"},
      {"data", QJsonObject{{"providerId", "emoji"},
                           {"key", key},
                           {"epoch", static_cast<qint64>(epoch)}}}};
  m_iface.call("RequestPreview", QString::fromUtf8(QJsonDocument(env).toJson(
                                     QJsonDocument::Compact)));
}

QString EngineProxy::execute(const QString &actionId, const QString &key) {
  const QJsonObject env{{"v", "1.0"},
                        {"data", QJsonObject{{"providerId", "emoji"},
                                             {"actionId", actionId},
                                             {"key", key}}}};
  QDBusReply<QString> reply = m_iface.call(
      "Execute",
      QString::fromUtf8(QJsonDocument(env).toJson(QJsonDocument::Compact)));
  return reply.isValid() ? reply.value() : QString();
}
