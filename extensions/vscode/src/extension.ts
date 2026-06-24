import * as cp from "node:child_process";
import * as vscode from "vscode";

type TreeNode = {
  label: string;
  description?: string;
  file?: string;
  line?: number;
  children?: TreeNode[];
};

class SbeTreeProvider implements vscode.TreeDataProvider<TreeNode> {
  private readonly changed = new vscode.EventEmitter<TreeNode | undefined>();
  readonly onDidChangeTreeData = this.changed.event;
  private roots: TreeNode[] = [];

  setRoots(roots: TreeNode[]) {
    this.roots = roots;
    this.changed.fire(undefined);
  }

  getTreeItem(element: TreeNode): vscode.TreeItem {
    const item = new vscode.TreeItem(
      element.label,
      element.children?.length
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None,
    );
    item.description = element.description;
    if (element.file) {
      item.command = {
        command: "vscode.open",
        title: "Open Symbol",
        arguments: [
          vscode.Uri.file(element.file),
          element.line
            ? { selection: new vscode.Range(element.line - 1, 0, element.line - 1, 0) }
            : undefined,
        ],
      };
    }
    return item;
  }

  getChildren(element?: TreeNode): TreeNode[] {
    return element?.children ?? this.roots;
  }
}

export function activate(context: vscode.ExtensionContext) {
  const provider = new SbeTreeProvider();
  context.subscriptions.push(vscode.window.registerTreeDataProvider("sbeResults", provider));

  context.subscriptions.push(
    vscode.commands.registerCommand("sbe.traceSymbol", async () => {
      const symbol = await promptSymbol();
      if (!symbol) return;
      const output = await runSbe(["trace", symbol, "--json"]);
      const traces = JSON.parse(output) as Array<{ root: string; path: string[] }>;
      provider.setRoots(
        traces.map((trace) => ({
          label: trace.root,
          children: trace.path.map((node) => ({ label: node })),
        })),
      );
    }),
    vscode.commands.registerCommand("sbe.showImpact", async () => {
      const symbol = await promptSymbol();
      if (!symbol) return;
      const reports = JSON.parse(await runSbe(["impact", symbol, "--json"])) as Array<{
        affected: Array<{ name: string; kind: string; range: { start_line: number } }>;
      }>;
      provider.setRoots(
        reports.map((report, index) => ({
          label: `Impact ${index + 1}`,
          children: report.affected.map((symbol) => ({
            label: symbol.name,
            description: symbol.kind,
            line: symbol.range.start_line,
          })),
        })),
      );
    }),
    vscode.commands.registerCommand("sbe.explainSymbol", async () => {
      const symbol = await promptSymbol();
      if (!symbol) return;
      const packets = JSON.parse(await runSbe(["inspect", symbol, "--json"])) as Array<{
        symbol: { name: string; kind: string };
        file_path: string;
        source_lines: [number, number];
        direct_dependencies: Array<{ name: string }>;
        dependents: Array<{ name: string }>;
      }>;
      provider.setRoots(
        packets.map((packet) => ({
          label: packet.symbol.name,
          description: packet.symbol.kind,
          file: packet.file_path,
          line: packet.source_lines[0],
          children: [
            {
              label: "Dependencies",
              children: packet.direct_dependencies.map((dependency) => ({
                label: dependency.name,
              })),
            },
            {
              label: "Dependents",
              children: packet.dependents.map((dependent) => ({ label: dependent.name })),
            },
          ],
        })),
      );
    }),
  );
}

export function deactivate() {}

async function promptSymbol(): Promise<string | undefined> {
  const editor = vscode.window.activeTextEditor;
  const selected = editor?.document.getText(editor.selection).trim();
  return vscode.window.showInputBox({
    prompt: "Symbol name",
    value: selected,
  });
}

async function runSbe(args: string[]): Promise<string> {
  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspace) {
    throw new Error("Open a workspace before running SBE.");
  }
  const binary = vscode.workspace.getConfiguration("sbe").get<string>("binaryPath", "sbe");
  return new Promise((resolve, reject) => {
    const child = cp.execFile(binary, [...args, workspace], { cwd: workspace }, (error, stdout, stderr) => {
      if (error) {
        reject(new Error(stderr || error.message));
        return;
      }
      resolve(stdout);
    });
    child.on("error", reject);
  });
}
