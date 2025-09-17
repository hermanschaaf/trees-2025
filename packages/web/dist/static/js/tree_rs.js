let wasm;

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedFloat32ArrayMemory0 = null;

function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedUint32ArrayMemory0 = null;

function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_0.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * @param {number} seed
 * @param {number} trunk_height
 * @param {number} butressing
 * @returns {TreeObject}
 */
export function generate(seed, trunk_height, butressing) {
    const ret = wasm.generate(seed, trunk_height, butressing);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return TreeObject.__wrap(ret[0]);
}

const QuaternionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_quaternion_free(ptr >>> 0, 1));

export class Quaternion {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        QuaternionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_quaternion_free(ptr, 0);
    }
    /**
     * @param {number} w
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(w, x, y, z) {
        const ret = wasm.quaternion_new(w, x, y, z);
        this.__wbg_ptr = ret >>> 0;
        QuaternionFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.quaternion_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.quaternion_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.quaternion_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.quaternion_z(this.__wbg_ptr);
        return ret;
    }
}

const TreeMeshFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_treemesh_free(ptr >>> 0, 1));

export class TreeMesh {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TreeMesh.prototype);
        obj.__wbg_ptr = ptr;
        TreeMeshFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TreeMeshFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_treemesh_free(ptr, 0);
    }
    /**
     * @returns {Float32Array}
     */
    get vertices() {
        const ret = wasm.treemesh_vertices(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    get normals() {
        const ret = wasm.treemesh_normals(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    get uvs() {
        const ret = wasm.treemesh_uvs(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    get indices() {
        const ret = wasm.treemesh_indices(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    get depths() {
        const ret = wasm.treemesh_depths(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}

const TreeObjectFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_treeobject_free(ptr >>> 0, 1));

export class TreeObject {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TreeObject.prototype);
        obj.__wbg_ptr = ptr;
        TreeObjectFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TreeObjectFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_treeobject_free(ptr, 0);
    }
    /**
     * @param {number} seed
     * @param {number} trunk_height
     * @param {number} butressing
     */
    constructor(seed, trunk_height, butressing) {
        const ret = wasm.treeobject_new(seed, trunk_height, butressing);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        TreeObjectFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    render() {
        wasm.treeobject_render(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    rings_count() {
        const ret = wasm.treeobject_rings_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} index
     * @returns {Vector3d | undefined}
     */
    ring_center(index) {
        const ret = wasm.treeobject_ring_center(this.__wbg_ptr, index);
        return ret === 0 ? undefined : Vector3d.__wrap(ret);
    }
    /**
     * @param {number} index
     * @returns {number | undefined}
     */
    ring_radius(index) {
        const ret = wasm.treeobject_ring_radius(this.__wbg_ptr, index);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @returns {number}
     */
    twigs_count() {
        const ret = wasm.treeobject_twigs_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} index
     * @returns {Vector3d | undefined}
     */
    twig_position(index) {
        const ret = wasm.treeobject_twig_position(this.__wbg_ptr, index);
        return ret === 0 ? undefined : Vector3d.__wrap(ret);
    }
    /**
     * @param {number} index
     * @returns {number | undefined}
     */
    twig_orientation_x(index) {
        const ret = wasm.treeobject_twig_orientation_x(this.__wbg_ptr, index);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number} index
     * @returns {number | undefined}
     */
    twig_orientation_y(index) {
        const ret = wasm.treeobject_twig_orientation_y(this.__wbg_ptr, index);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number} index
     * @returns {number | undefined}
     */
    twig_orientation_z(index) {
        const ret = wasm.treeobject_twig_orientation_z(this.__wbg_ptr, index);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number} index
     * @returns {number | undefined}
     */
    twig_orientation_w(index) {
        const ret = wasm.treeobject_twig_orientation_w(this.__wbg_ptr, index);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number} index
     * @returns {number | undefined}
     */
    twig_scale(index) {
        const ret = wasm.treeobject_twig_scale(this.__wbg_ptr, index);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number} index
     * @returns {string | undefined}
     */
    twig_type(index) {
        const ret = wasm.treeobject_twig_type(this.__wbg_ptr, index);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @param {number} resolution
     * @returns {TreeMesh}
     */
    generate_tree_mesh(resolution) {
        const ret = wasm.treeobject_generate_tree_mesh(this.__wbg_ptr, resolution);
        return TreeMesh.__wrap(ret);
    }
    /**
     * @param {number} height
     */
    set_trunk_height(height) {
        wasm.treeobject_set_trunk_height(this.__wbg_ptr, height);
    }
    /**
     * @param {number} butressing
     */
    set_butressing(butressing) {
        wasm.treeobject_set_butressing(this.__wbg_ptr, butressing);
    }
    /**
     * @param {number} split_height
     */
    set_split_height(split_height) {
        wasm.treeobject_set_split_height(this.__wbg_ptr, split_height);
    }
    /**
     * @param {number} segment_length
     */
    set_segment_length(segment_length) {
        wasm.treeobject_set_segment_length(this.__wbg_ptr, segment_length);
    }
    /**
     * @param {number} min
     * @param {number} max
     */
    set_branch_angle_range(min, max) {
        wasm.treeobject_set_branch_angle_range(this.__wbg_ptr, min, max);
    }
    /**
     * @param {number} min
     * @param {number} max
     */
    set_bend_angle_range(min, max) {
        wasm.treeobject_set_bend_angle_range(this.__wbg_ptr, min, max);
    }
    /**
     * @param {number} min
     * @param {number} max
     */
    set_branch_frequency_range(min, max) {
        wasm.treeobject_set_branch_frequency_range(this.__wbg_ptr, min, max);
    }
    /**
     * @param {number} max_depth
     */
    set_max_depth(max_depth) {
        wasm.treeobject_set_max_depth(this.__wbg_ptr, max_depth);
    }
    /**
     * @param {number} radius_taper
     */
    set_radius_taper(radius_taper) {
        wasm.treeobject_set_radius_taper(this.__wbg_ptr, radius_taper);
    }
    /**
     * @param {number} trunk_ring_spread
     */
    set_trunk_ring_spread(trunk_ring_spread) {
        wasm.treeobject_set_trunk_ring_spread(this.__wbg_ptr, trunk_ring_spread);
    }
    /**
     * @param {number} segment_length_variation
     */
    set_segment_length_variation(segment_length_variation) {
        wasm.treeobject_set_segment_length_variation(this.__wbg_ptr, segment_length_variation);
    }
    /**
     * @param {number} trunk_size
     */
    set_trunk_size(trunk_size) {
        wasm.treeobject_set_trunk_size(this.__wbg_ptr, trunk_size);
    }
    /**
     * @param {number} variation
     */
    set_branch_azimuth_variation(variation) {
        wasm.treeobject_set_branch_azimuth_variation(this.__wbg_ptr, variation);
    }
    /**
     * @param {number} reach
     */
    set_max_branch_reach(reach) {
        wasm.treeobject_set_max_branch_reach(this.__wbg_ptr, reach);
    }
    /**
     * @param {boolean} enable
     */
    set_root_enable(enable) {
        wasm.treeobject_set_root_enable(this.__wbg_ptr, enable);
    }
    /**
     * @param {number} depth
     */
    set_root_depth(depth) {
        wasm.treeobject_set_root_depth(this.__wbg_ptr, depth);
    }
    /**
     * @param {number} spread
     */
    set_root_spread(spread) {
        wasm.treeobject_set_root_spread(this.__wbg_ptr, spread);
    }
    /**
     * @param {number} density
     */
    set_root_density(density) {
        wasm.treeobject_set_root_density(this.__wbg_ptr, density);
    }
    /**
     * @param {number} segment_length
     */
    set_root_segment_length(segment_length) {
        wasm.treeobject_set_root_segment_length(this.__wbg_ptr, segment_length);
    }
    /**
     * @param {boolean} enable
     */
    set_twig_enable(enable) {
        wasm.treeobject_set_twig_enable(this.__wbg_ptr, enable);
    }
    /**
     * @param {number} density
     */
    set_twig_density(density) {
        wasm.treeobject_set_twig_density(this.__wbg_ptr, density);
    }
    /**
     * @param {number} scale
     */
    set_twig_scale(scale) {
        wasm.treeobject_set_twig_scale(this.__wbg_ptr, scale);
    }
    /**
     * @param {number} variation
     */
    set_twig_angle_variation(variation) {
        wasm.treeobject_set_twig_angle_variation(this.__wbg_ptr, variation);
    }
    /**
     * Export the tree as a GLTF file (returns JSON as string)
     * @param {number} resolution
     * @returns {string}
     */
    export_gltf(resolution) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.treeobject_export_gltf(this.__wbg_ptr, resolution);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
}

const Vector3dFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_vector3d_free(ptr >>> 0, 1));

export class Vector3d {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Vector3d.prototype);
        obj.__wbg_ptr = ptr;
        Vector3dFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Vector3dFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_vector3d_free(ptr, 0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.vector3d_new(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        Vector3dFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.quaternion_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.quaternion_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.quaternion_y(this.__wbg_ptr);
        return ret;
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_0;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedFloat32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('tree_rs_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
