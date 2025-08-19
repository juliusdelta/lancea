#pragma once
#include <QDBusConnection>
#include <QDBusInterface>
#include <QObject>

class EngineProxy : public QObject {
  Q_OBJECT
public:
  explicit EngineProxy(QObject *parent = nullptr);

  Q_INVOKABLE QString resolveCommand(const QString &text);
  Q_INVOKABLE quint64 search(const QString &text, const QString &providerId, quint64 epoch = 0);
  Q_INVOKABLE void requestPreview(const QString &key, quint64 epoch = 0);
  Q_INVOKABLE QString execute(const QString &action, const QString &providerId, const QString &key);

signals:
  void resultsUpdated(qulonglong epoch, QString providerId, qulonglong token,
                      QString batchJson);
  void previewUpdated(qulonglong epoch, QString providerId, QString resultKey,
                      QString previewJson);
  void providerError(qulonglong epoch, QString providerId, QString errJson);

private slots:
  // These receive D-Bus signals and re-emit the Qt signals above
  void handleResultsUpdated(qulonglong epoch, const QString &providerId,
                            qulonglong token, const QString &batchJson);
  void handlePreviewUpdated(qulonglong epoch, const QString &providerId,
                            const QString &resultKey,
                            const QString &previewJson);
  void handleProviderError(qulonglong epoch, const QString &providerId,
                           const QString &errJson);

private:
  QDBusInterface m_iface;
};
