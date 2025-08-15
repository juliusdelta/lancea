#pragma once
#include <QDBusConnection>
#include <QDBusInterface>
#include <QObject>

class EngineProxy : public QObject {
  Q_OBJECT
public:
  explicit EngineProxy(QObject *parent = nullptr);

  Q_INVOKABLE QString resolveCommand(const QString &text);
  Q_INVOKABLE quint64 search(const QString &text, quint64 epoch = 0);
  Q_INVOKABLE void requestPreview(const QString &key, quint64 epoch = 0);
  Q_INVOKABLE QString execute(const QString &actionId, const QString &key);

signals:
  void ResultsUpdated(qulonglong epoch, QString providerId, qulonglong token,
                      QString batchJson);
  void PreviewUpdated(qulonglong epoch, QString providerId, QString resultKey,
                      QString previewJson);
  void ProviderError(qulonglong epoch, QString providerId, QString errJson);

private:
  QDBusInterface m_iface;
};
