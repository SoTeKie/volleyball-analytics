import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { TextField, TextFieldRoot } from "@/components/ui/textfield";

type ParseResult = {
  teamScored: "Away" | "Home";
};

type ParseError = {
  key: "WhoScored" | "InvalidInput";
  isParsingError: true
};

function isParseError(error: any): error is ParseError {
  return (error as ParseError).isParsingError === true;
}

function App() {
  const [rally, setRally] = createSignal("");
  const [rallyResult, setRallyResult] = createSignal<ParseResult | null>(null);
  const [parsingError, setParsingError] = createSignal<ParseError | null>(null);

  const parseRally = async () => {
    try {
      setRallyResult(await invoke("parse_rally", { rally: rally() }));
      setParsingError(null);
    } catch (e) {
      console.log(e);
      if (isParseError(e)) {
        setRallyResult(null);
        setParsingError(e);
        return;
      }
      
      throw e;
    }
  }

  const handleSubmit = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      parseRally()
    }
  }

  return (
    <div>
      <h1 class="text-2xl">Welcome to Volleyball Analytics!</h1>
      <TextFieldRoot
        value={rally()}
        onChange={setRally}
        onKeyPress={handleSubmit}
      >
        <TextField />
      </TextFieldRoot>
      <Show when={parsingError() !== null}>
        <p>{parsingError()?.key}</p>
      </Show>
      <Show when={rallyResult() !== null}>
        <p>{rallyResult()?.teamScored}</p>
      </Show>
    </div>
  );
}

export default App;
