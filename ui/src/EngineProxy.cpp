#include "EngineProxy.h"
#include <QDBusReply>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>

static const char *SVC = "org.lancea.Engine1";
static const char *PATH = "/org/lancea/Engine1";
static const char *IFACE = "org.lancea.Engine1";

EngineProxy::EngineProxy(QObject *parent)
    : QObject(parent),
      m_iface(SVC, PATH, IFACE, QDBusConnection::sessionBus(), this) {

  bool ok = true;
  ok &= QDBusConnection::sessionBus().connect(
      SVC, PATH, IFACE, "ResultsUpdated", this,
      SLOT(handleResultsUpdated(qulonglong, QString, qulonglong, QString)));
  ok &= QDBusConnection::sessionBus().connect(
      SVC, PATH, IFACE, "PreviewUpdated", this,
      SLOT(handlePreviewUpdated(qulonglong, QString, QString, QString)));
  ok &= QDBusConnection::sessionBus().connect(
      SVC, PATH, IFACE, "ProviderError", this,
      SLOT(handleProviderError(qulonglong, QString, QString)));
  if (!ok) {
    qWarning() << "EngineProxy: one or more DBus signal connections failed";
  }
}

void EngineProxy::handleResultsUpdated(qulonglong epoch,
                                       const QString &providerId,
                                       qulonglong token,
                                       const QString &batchJson) {
  emit resultsUpdated(epoch, providerId, token, batchJson);
}

void EngineProxy::handlePreviewUpdated(qulonglong epoch,
                                       const QString &providerId,
                                       const QString &resultKey,
                                       const QString &previewJson) {
  emit previewUpdated(epoch, providerId, resultKey, previewJson);
}

void EngineProxy::handleProviderError(qulonglong epoch,
                                      const QString &providerId,
                                      const QString &errJson) {
  emit providerError(epoch, providerId, errJson);
}

QString EngineProxy::resolveCommand(const QString &text) {
  const QJsonObject env{{"v", "1.0"}, {"data", QJsonObject{{"text", text}}}};
  QDBusReply<QString> reply = m_iface.call(
      "ResolveCommand",
      QString::fromUtf8(QJsonDocument(env).toJson(QJsonDocument::Compact)));

  return reply.isValid() ? reply.value() : QString();
}

quint64 EngineProxy::search(const QString &text, const QString &providerId,
                            quint64 epoch) {
  QJsonObject data;
  data.insert(QStringLiteral("text"), text);

  QJsonArray providerIds;
  providerIds.append(providerId);
  data.insert(QStringLiteral("providerIds"), providerIds);

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

QString EngineProxy::execute(const QString &action, const QString &key) {
  const QJsonObject env{
      {"v", "1.0"},
      {"data",
       QJsonObject{{"providerId", "emoji"}, {"action", action}, {"key", key}}}};
  QDBusReply<QString> reply = m_iface.call(
      "Execute",
      QString::fromUtf8(QJsonDocument(env).toJson(QJsonDocument::Compact)));
  return reply.isValid() ? reply.value() : QString();
}
