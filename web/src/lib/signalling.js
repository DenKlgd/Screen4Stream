/**
 * Зритель WebRTC-трансляции.
 *
 * Говорит на signaling-протоколе webrtcsink (gst-plugins-rs). Сервер протокола
 * запущен внутри нашего приложения: crates/server/src/signalling.rs.
 *
 * Как устанавливается соединение:
 *  1. Подключаемся по WebSocket и регистрируемся слушателем (listener) —
 *     так сервер сообщит нам, когда трансляция появится или пропадёт.
 *  2. Запрашиваем список продюсеров (list) и начинаем сессию с первым (startSession).
 *  3. webrtcsink присылает offer (описание потоков, SDP), мы отвечаем answer.
 *  4. Обе стороны обмениваются ICE-кандидатами (адресами), после чего
 *     видео идёт напрямую от компьютера ведущего к браузеру.
 *
 * Состояния, которые получает onState:
 *   connecting — подключаемся к серверу или к трансляции
 *   waiting    — сервер доступен, но трансляции нет
 *   live       — видео идёт
 *   ended      — ведущий остановил трансляцию (ждём следующую)
 *   offline    — нет связи с компьютером ведущего (переподключаемся)
 *   failed     — WebRTC-соединение не установилось (часто мешает брандмауэр)
 */
export function connectViewer({ url, onStream, onState }) {
  let ws = null;
  let pc = null;
  let sessionId = null;
  let starting = false;
  let wasLive = false;
  let closed = false;
  let retryTimer = null;
  // Сообщения обрабатываем строго по очереди: ICE-кандидат нельзя
  // добавить, пока не применён offer.
  let queue = Promise.resolve();

  function send(message) {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message));
    }
  }

  function connect() {
    onState('connecting');
    ws = new WebSocket(url);
    ws.onopen = () => {
      send({ type: 'setPeerStatus', roles: ['listener'], meta: null });
      send({ type: 'list' });
    };
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      queue = queue.then(() => handle(message)).catch((err) => console.error('signaling:', err));
    };
    ws.onclose = () => {
      resetSession();
      if (!closed) {
        onState('offline');
        retryTimer = setTimeout(connect, 2000);
      }
    };
  }

  function startSession(producerId) {
    if (sessionId || starting) return;
    starting = true;
    onState('connecting');
    send({ type: 'startSession', peerId: producerId });
  }

  async function handle(message) {
    switch (message.type) {
      case 'list':
        if (message.producers.length > 0) {
          startSession(message.producers[0].id);
        } else {
          onState(wasLive ? 'ended' : 'waiting');
        }
        break;

      case 'peerStatusChanged':
        // Появился продюсер — например, ведущий начал трансляцию или сменил источник.
        if (message.roles.includes('producer')) {
          startSession(message.peerId);
        }
        break;

      case 'sessionStarted':
        sessionId = message.sessionId;
        starting = false;
        wasLive = true;
        break;

      case 'peer':
        if (message.sessionId === sessionId) {
          await handlePeerMessage(message);
        }
        break;

      case 'endSession':
        if (message.sessionId === sessionId) {
          resetSession();
          onState('ended');
          // При смене источника новый продюсер успевает зарегистрироваться раньше,
          // чем закроется старая сессия, и его peerStatusChanged мы пропустили.
          // Поэтому переспрашиваем список: если трансляция уже есть — подключимся.
          send({ type: 'list' });
        }
        break;

      case 'error':
        console.warn('signaling:', message.details);
        break;
    }
  }

  async function handlePeerMessage(message) {
    if (message.sdp?.type === 'offer') {
      createPeerConnection();
      await pc.setRemoteDescription(message.sdp);
      await pc.setLocalDescription(await pc.createAnswer());
      send({ type: 'peer', sessionId, sdp: { type: 'answer', sdp: pc.localDescription.sdp } });
    } else if (message.ice && pc) {
      const { candidate, sdpMLineIndex } = message.ice;
      await pc.addIceCandidate({ candidate, sdpMLineIndex }).catch(() => {});
    }
  }

  function createPeerConnection() {
    pc?.close();
    // В локальной сети STUN/TURN-серверы не нужны: адреса видны напрямую.
    pc = new RTCPeerConnection();
    const stream = new MediaStream();

    pc.ontrack = (event) => {
      stream.addTrack(event.track);
      onStream(stream);
    };
    pc.onicecandidate = (event) => {
      if (event.candidate) {
        send({
          type: 'peer',
          sessionId,
          ice: { candidate: event.candidate.candidate, sdpMLineIndex: event.candidate.sdpMLineIndex },
        });
      }
    };
    pc.onconnectionstatechange = () => {
      if (pc?.connectionState === 'connected') onState('live');
      if (pc?.connectionState === 'failed') onState('failed');
    };
  }

  function resetSession() {
    pc?.close();
    pc = null;
    sessionId = null;
    starting = false;
  }

  /** Статистика входящего видео для оверлея; null, если соединения нет. */
  async function getStats() {
    if (!pc) return null;
    const report = await pc.getStats();
    let video = null;
    let pair = null;
    report.forEach((s) => {
      if (s.type === 'inbound-rtp' && s.kind === 'video') video = s;
      if (s.type === 'candidate-pair' && s.nominated && s.state === 'succeeded') pair = s;
    });
    if (!video) return null;
    const codec = video.codecId ? report.get(video.codecId) : null;
    return {
      width: video.frameWidth,
      height: video.frameHeight,
      fps: video.framesPerSecond,
      bytesReceived: video.bytesReceived,
      timestamp: video.timestamp,
      packetsLost: video.packetsLost ?? 0,
      packetsReceived: video.packetsReceived ?? 0,
      codec: codec?.mimeType?.replace('video/', ''),
      rttMs: pair?.currentRoundTripTime != null ? pair.currentRoundTripTime * 1000 : null,
      jitterBufferMs: video.jitterBufferEmittedCount
        ? (video.jitterBufferDelay / video.jitterBufferEmittedCount) * 1000
        : null,
    };
  }

  /** Переподключиться вручную (кнопка «Попробовать снова»). */
  function retry() {
    clearTimeout(retryTimer);
    resetSession();
    if (ws?.readyState === WebSocket.OPEN) {
      send({ type: 'list' });
    } else {
      ws?.close();
      connect();
    }
  }

  function close() {
    closed = true;
    clearTimeout(retryTimer);
    if (sessionId) send({ type: 'endSession', sessionId });
    resetSession();
    ws?.close();
  }

  connect();
  return { getStats, retry, close };
}
