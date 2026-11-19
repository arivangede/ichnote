import { useEffect, useState } from "react";
import { getAppInfo } from "../lib/ipc";

export function MainWindow() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getAppInfo()
      .then((info) => setVersion(info.version))
      .catch(() => setVersion(null));
  }, []);

  return (
    <main className="placeholder">
      <h1>Ichnote</h1>
      <p>Leave a trace.</p>
      {version && <p className="version">v{version}</p>}
    </main>
  );
}
