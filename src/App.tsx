import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { TextField, TextFieldRoot } from "@/components/ui/textfield";

interface ParseResult {
  Ok?: Stats;
  Fail?: Reason;
};

interface PlayerScores {
  scored: number;
  faults: number;
  all: number;
}

interface PlayerStats {
  player: number,
  hits: PlayerScores,
  blocks: PlayerScores,
  serves: PlayerScores,
}

interface StatsByPlayer {
  [key: string]: PlayerStats;
}

interface TeamStats {
  sets: number;
  points: number;
  playerStats: StatsByPlayer
}

interface Stats {
  awayTeam: TeamStats;
  homeTeam: TeamStats;
  status: "InProgress" | "Finished";
}

interface Reason {
  errorMsg: string;
  location: number;
};

const initialStats: Stats = {
  awayTeam: {
    sets: 0,
    points: 0,
    playerStats: {},
  },
  homeTeam: {
    sets: 0,
    points: 0,
    playerStats: {},
  },
  status: "InProgress",
}

function App() {
  const [rally, setRally] = createSignal("");
  const [matchState, setMatchState] = createSignal<Stats>(initialStats);
  const [failReason, setFailReason] = createSignal<Reason>();

  const parseRally = async () => {
    const result = await invoke<ParseResult>(
      "parse_rally",
      { currentStats: matchState(), rally: rally() }
    );

    console.log(result);

    if (result.Ok !== undefined) {
      setMatchState(result.Ok);
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
        <span class="rounded p-1 text-xl bg-gray-900 text-white">{matchState().homeTeam.sets}</span>
        <span class="rounded p-1 text-xl bg-red-600 text-white">{matchState().homeTeam.points}</span>
        <span class="rounded p-1 text-xl">-</span>
        <span class="rounded p-1 text-xl bg-red-600 text-white">{matchState().awayTeam.points}</span>
        <span class="rounded p-1 text-xl bg-gray-900 text-white">{matchState().awayTeam.sets}</span>
      </div>
      <TextFieldRoot
        value={rally()}
        onChange={setRally}
        disabled={matchState().status == "Finished"}
        onKeyPress={handleSubmit}
        class="w-3/6"
      >
        <TextField />
      </TextFieldRoot>
      <Show when={matchState().status == "Finished"}>
        <p class="text-green-900">MATCH FINISHED</p>
      </Show>
      <Show when={failReason()}>
        {(r) => <p class="text-destructive">{r().errorMsg}</p>}
      </Show>
    </div>
  );
}

export default App;
