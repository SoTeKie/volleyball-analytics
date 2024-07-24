import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { TextField, TextFieldRoot } from "@/components/ui/textfield";

interface ParseResult {
  Ok?: Stats;
  Fail?: Reason;
};

interface TeamStats {
  sets: number;
  points: number;
}

interface Stats {
  awayTeam: TeamStats;
  homeTeam: TeamStats;
}

interface Reason {
  key: "WhoScored" | "InvalidInput";
  errorMsg: string;
};

const initialStats: Stats = {
  awayTeam: {
    sets: 0,
    points: 0,
  },
  homeTeam: {
    sets: 0,
    points: 0,
  },
}

function App() {
  const [rally, setRally] = createSignal("");
  const [stats, setStats] = createSignal<Stats>(initialStats);
  const [failReason, setFailReason] = createSignal<Reason>();

  const parseRally = async () => {
    const result = await invoke<ParseResult>(
      "parse_rally",
      { currentStats: stats(), rally: rally() }
    );

    console.log(result);

    if (result.Ok !== undefined) {
      setStats(result.Ok);
    }
    setFailReason(result.Fail);
  }

  const handleSubmit = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      parseRally()
    }
  }

  return (
    <div class="flex flex-col items-center justify-center h-dvh gap-4">
      <h1 class="text-xl">Welcome to Volleyball Analytics!</h1>
      <div class="flex flex-row gap-4">
        <span class="rounded p-1 text-xl bg-gray-900 text-white">{stats().homeTeam.sets}</span>
        <span class="rounded p-1 text-xl bg-red-600 text-white">{stats().homeTeam.points}</span>
        <span class="rounded p-1 text-xl">-</span>
        <span class="rounded p-1 text-xl bg-red-600 text-white">{stats().awayTeam.points}</span>
        <span class="rounded p-1 text-xl bg-gray-900 text-white">{stats().awayTeam.sets}</span>
      </div>
      <TextFieldRoot
        value={rally()}
        onChange={setRally}
        onKeyPress={handleSubmit}
        class="w-3/6"
      >
        <TextField />
      </TextFieldRoot>
      <Show when={failReason()}>
        {(r) => <p class="text-destructive">{r().errorMsg}</p>}
      </Show>
    </div>
  );
}

export default App;
