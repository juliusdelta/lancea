#pragma once
#include <QClipboard>
#include <QGuiApplication>
#include <QObject>

class ClipboardProxy : public QObject {
  Q_OBJECT
public:
  explicit ClipboardProxy(QObject *parent = nullptr)
      : QObject(parent), m_clipboard(QGuiApplication::clipboard()) {
    // Forward native change notification to QML
    connect(m_clipboard, &QClipboard::changed, this,
            [this](QClipboard::Mode mode) {
              if (mode == QClipboard::Clipboard)
                emit changed();
#ifdef Q_OS_LINUX
      // If you also want PRIMARY selection (middle-click paste) support,
      // uncomment: if (mode == QClipboard::Selection) emit selectionChanged();
#endif
            });
  }

  Q_INVOKABLE QString getText() const {
    return m_clipboard->text(QClipboard::Clipboard);
  }

  Q_INVOKABLE void setText(const QString &text) {
    m_clipboard->setText(text, QClipboard::Clipboard);
    // Emit so QML bindings / tests can react immediately
    emit changed();
  }

  Q_INVOKABLE void clear() {
    m_clipboard->clear(QClipboard::Clipboard);
    emit changed();
  }

#ifdef Q_OS_LINUX
  // Optional: X11/Wayland PRIMARY selection helpers
  Q_INVOKABLE QString getPrimarySelection() const {
    return m_clipboard->text(QClipboard::Selection);
  }
  Q_INVOKABLE void setPrimarySelection(const QString &text) {
    m_clipboard->setText(text, QClipboard::Selection);
    emit selectionChanged();
  }
#endif

signals:
  void changed();
#ifdef Q_OS_LINUX
  void selectionChanged();
#endif

private:
  QClipboard *m_clipboard;
};
