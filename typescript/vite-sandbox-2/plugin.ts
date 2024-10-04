import { Plugin, OutputChunk, OutputOptions, ManualChunksOption, GetModuleInfo, OutputBundle } from 'rollup';
import { createHash } from 'crypto';

interface SplitChunksOptions {
  minSize: number;            // チャンクの最小サイズ
  maxSize: number;            // チャンクの最大サイズ
  minChunks: number;          // チャンクとして分割されるための最小参照回数
}

// copy from constants.ts
const CSS_LANGS_RE =
  /\.(css|less|sass|scss|styl|stylus|pcss|postcss|sss)(?:$|\?)/
export const isCSSRequest = (request: string): boolean =>
  CSS_LANGS_RE.test(request)

export function splitChunksPlugin(options: SplitChunksOptions): Plugin {
  const moduleSizeMap = new Map<string, number>();   // モジュールのサイズを追跡
  const moduleReferenceMap = new Map<string, number>(); // モジュールの参照回数を追跡

  return {
    name: 'vite-plugin-split-chunks',

    outputOptions(outputOptions: OutputOptions) {
      const manualChunks: ManualChunksOption = (id: string, { getModuleInfo }) => {
        const moduleInfo = getModuleInfo(id);
        if (!moduleInfo) return null;

        // モジュールのサイズを計算
        const moduleSize = Buffer.byteLength(moduleInfo.code || '', 'utf8');
        moduleSizeMap.set(id, moduleSize);


        // 再帰的に依存モジュールの参照回数をカウント
        const referenceCount = getRecursiveReferenceCount(id, getModuleInfo, new Map());
        moduleReferenceMap.set(id, referenceCount);

        // `node_modules` 内のモジュールは vendor チャンクに分割
        if (id.includes('node_modules')) {
          const packageName = id.split('node_modules/')[1].split('/')[0];
          return `vendor-${packageName}`;
        }

        // `minChunks` のロジック: 参照回数が minChunks を超えた場合、共通チャンクとして分割
        if (referenceCount >= options.minChunks) {
          return `common-chunk-${getModuleHash(id)}`;
        }

        return null;
      };

      return {
        ...outputOptions,
        manualChunks,
      };
    },

    buildStart() {
      // モジュール情報をリセット
      moduleSizeMap.clear();
      moduleReferenceMap.clear();
    },

    transform(code, id) {
      // モジュールのサイズを計算し、サイズマップに追加
      const moduleSize = Buffer.byteLength(code, 'utf8');
      moduleSizeMap.set(id, moduleSize);
    },

    generateBundle(_, bundle) {
      // minSizeを満たさないチャンクを統合
      for (const [fileName, chunk] of Object.entries(bundle)) {
        if (!chunk.name.includes('vendor') && !chunk.name.includes('common-chunk')) {
          continue
        }
        if (chunk.type === 'chunk') {
          const chunkSize = Buffer.byteLength(chunk.code, 'utf8');

          if (chunkSize < options.minSize) {
            mergeSmallChunks(chunk, bundle);
          }

          if (chunkSize > options.maxSize) {
            splitLargeChunk(chunk, bundle, options.maxSize);
          }

        }
      }
    },
  };

  // モジュールのハッシュを生成する関数
  function getModuleHash(id: string): string {
    return createHash('sha256').update(id).digest('hex').slice(0, 8);
  }

  // チャンクが maxSize を超えた場合の分割ロジック
  function splitLargeChunk(chunk: OutputChunk, bundle: OutputBundle, maxSize: number) {
    // 使用するdeterministicGroupingアルゴリズムを利用してチャンクを分割する
    const items = Object.keys(chunk.modules).map((id) => ({
      id,
      size: moduleSizeMap.get(id) || 0,
      key: id
    }));

    const groupedItems = deterministicGrouping({
      maxSize: { 'javascript': maxSize },
      minSize: { 'javascript': options.minSize },
      items,
      getSize: (item) => ({ 'javascript': item.size }),
      getKey: (item) => item.key
    });

    // チャンク分割後のチェックと削除のためのリスト
    const chunksToDelete = [];

    // 元のチャンクファイル名の拡張子を取得
    const originalExtension = chunk.fileName.split('.').pop() || 'js';

    groupedItems.forEach((group, index) => {
      // 新しいチャンクを作成
      const newChunkModules = group.items.reduce((acc, item) => {
        acc[item.id] = chunk.modules[item.id];
        return acc;
      }, {} as Record<string, any>);

      // 新しいチャンクのcodeを生成する
      const newCode = group.items.map(item => chunk.modules[item.id].code).join('\n');
      const newSize = Buffer.byteLength(newCode, 'utf8');

      // チャンクのサイズが 0 ではない場合のみ新しいチャンクを作成する
      if (newSize > 0) {
        const newChunk: OutputChunk = {
          ...chunk,
          fileName: `${chunk.fileName.replace(/\.[^.]+$/, '')}-part${index}.${originalExtension}`,
          modules: newChunkModules,
          code: newCode
        };

        // バンドルに追加
        bundle[newChunk.fileName] = newChunk;

        // 移動済みのモジュールを元のチャンクから削除
        group.items.forEach(item => {
          delete chunk.modules[item.id];
        });
      } else {
        // サイズが0のものは削除対象としてリストに追加
        chunksToDelete.push(`${chunk.fileName.replace(/\.[^.]+$/, '')}-part${index}.${originalExtension}`);
      }
    });

    // 元のチャンクが空になった場合は削除
    if (Object.keys(chunk.modules).length === 0) {
      chunksToDelete.push(chunk.fileName);
    }

    // バンドルからサイズ0のチャンクを削除
    chunksToDelete.forEach(fileName => {
      delete bundle[fileName];
    });
  }


  // 小さいチャンクをエントリーポイントに統合
  function mergeSmallChunks(chunk: OutputChunk, bundle: OutputBundle) {
    // チャンクが minSize 未満の場合にエントリチャンクに統合
    const entryChunk = Object.values(bundle).find(chunk => chunk.type === 'chunk' && chunk.isEntry) as OutputChunk;
    if (!entryChunk) return;

    Object.keys(chunk.modules).forEach(moduleId => {
      entryChunk.modules[moduleId] = chunk.modules[moduleId];
    });

    entryChunk.code = Object.values(entryChunk.modules).map(module => module.code).join('\n');
    delete bundle[chunk.fileName];
  }

  function getRecursiveReferenceCount(id: string, getModuleInfo: GetModuleInfo, cache: Map<string, number>): number {
    if (cache.has(id)) {
      return cache.get(id) || 0;
    }

    const moduleInfo = getModuleInfo(id);
    if (!moduleInfo) {
      cache.set(id, 0);
      return 0;
    }

    if (moduleInfo.isEntry) {
      cache.set(id, 1);
      return 1;
    }

    // 参照回数を計算
    let referenceCount = 0;
    moduleInfo.importers.forEach((importer) => {
      if (!cache.has(importer)) {
        referenceCount += getRecursiveReferenceCount(importer, getModuleInfo, cache);
      }
    });

    cache.set(id, referenceCount);
    return referenceCount;
  }

}

  class Node<T> {
    item: T;
    key: string;
    size: Record<string, number>;

    constructor(item: T, key: string, size: Record<string, number>) {
      this.item = item;
      this.key = key;
      this.size = size;
    }
  }

  class Group<T> {
    nodes: Node<T>[];
    similarities: number[] | null;
    size: Record<string, number>;
    key: string | undefined;

    constructor(nodes: Node<T>[], similarities: number[] | null, size?: Record<string, number>) {
      this.nodes = nodes;
      this.similarities = similarities;
      this.size = size || sumSize(nodes);
    }
  }


// webpack の deterministicGrouping 関数を Vite 用に変更
function deterministicGrouping<T>({
  maxSize,
  minSize,
  items,
  getSize,
  getKey
}: {
  maxSize: Record<string, number>,
  minSize: Record<string, number>,
  items: T[],
  getSize: (item: T) => Record<string, number>,
  getKey: (item: T) => string
}): { key: string, items: T[], size: Record<string, number> }[] {
  const nodes = items.map(item => new Node(item, getKey(item), getSize(item)));
  nodes.sort((a, b) => a.key.localeCompare(b.key));

  const result: Group<T>[] = [];


  // 初期ノードのフィルタリング
  const initialNodes: Node<T>[] = [];
  for (const node of nodes) {
    if (isTooBig(node.size, maxSize) && !isTooSmall(node.size, minSize)) {
      result.push(new Group([node], []));
    } else {
      initialNodes.push(node);
    }
  }

  if (initialNodes.length > 0) {
    const initialGroup = new Group(initialNodes, getSimilarities(initialNodes));
    const queue = [initialGroup];

    while (queue.length > 0) {
      const group = queue.pop()!;
      if (!isTooBig(group.size, maxSize)) {
        result.push(group);
        continue;
      }

      let left = 1;
      const leftSize = { ...group.nodes[0].size };
      while (left < group.nodes.length && isTooSmall(leftSize, minSize)) {
        addSizeTo(leftSize, group.nodes[left].size);
        left++;
      }

      let right = group.nodes.length - 2;
      const rightSize = { ...group.nodes[group.nodes.length - 1].size };
      while (right >= 0 && isTooSmall(rightSize, minSize)) {
        addSizeTo(rightSize, group.nodes[right].size);
        right--;
      }

      if (left - 1 > right) {
        result.push(group);
        continue;
      }

      const rightNodes = group.nodes.slice(right + 1);
      queue.push(new Group(rightNodes, getSimilarities(rightNodes)));

      const leftNodes = group.nodes.slice(0, left);
      queue.push(new Group(leftNodes, getSimilarities(leftNodes)));
    }
  }

  const usedNames = new Set<string>();
  return result.map(group => {
    const first = group.nodes[0];
    const last = group.nodes[group.nodes.length - 1];
    group.key = getName(first.key, last.key, usedNames);
    return { key: group.key, items: group.nodes.map(node => node.item), size: group.size };
  });
}

// サイズの合計を計算する関数
function sumSize<T>(nodes: Node<T>[]): Record<string, number> {
  const sum: Record<string, number> = {};
  nodes.forEach(node => addSizeTo(sum, node.size));
  return sum;
}

// 類似度を計算する関数
function getSimilarities<T>(nodes: Node<T>[]): number[] {
  const similarities: number[] = [];
  let previousKey: string | undefined;

  nodes.forEach(node => {
    if (previousKey !== undefined) {
      similarities.push(similarity(previousKey, node.key));
    }
    previousKey = node.key;
  });

  return similarities;
}

// 2つのサイズを比較して、`size` が `maxSize` を超えているかをチェック
function isTooBig(size: Record<string, number>, maxSize: Record<string, number>): boolean {
  for (const key of Object.keys(size)) {
    if (size[key] > maxSize[key]) {
      return true;
    }
  }
  return false;
}

// 2つのサイズを比較して、`size` が `minSize` を満たしているかをチェック
function isTooSmall(size: Record<string, number>, minSize: Record<string, number>): boolean {
  for (const key of Object.keys(size)) {
    if (size[key] < minSize[key]) {
      return true;
    }
  }
  return false;
}

// サイズを合計に加算する関数
function addSizeTo(target: Record<string, number>, size: Record<string, number>) {
  for (const key of Object.keys(size)) {
    target[key] = (target[key] || 0) + size[key];
  }
}

// 2つのキーの類似度を計算する関数
function similarity(a: string, b: string): number {
  const length = Math.min(a.length, b.length);
  let dist = 0;
  for (let i = 0; i < length; i++) {
    const ca = a.charCodeAt(i);
    const cb = b.charCodeAt(i);
    dist += Math.max(0, 10 - Math.abs(ca - cb));
  }
  return dist;
}

// 名前を生成する関数
function getName(a: string, b: string, usedNames: Set<string>): string {
  const l = Math.min(a.length, b.length);
  let i = 0;
  while (i < l) {
    if (a.charCodeAt(i) !== b.charCodeAt(i)) {
      i++;
      break;
    }
    i++;
  }
  while (i < l) {
    const name = a.slice(0, i);
    const lowerName = name.toLowerCase();
    if (!usedNames.has(lowerName)) {
      usedNames.add(lowerName);
      return name;
    }
    i++;
  }
  return a;
}