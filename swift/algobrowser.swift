import Foundation
import AudioToolbox

// Native Swift representation of the Rust-provided parameter
struct AlgoParam {
    let key: String
    let name: String
    let min: Float
    let max: Float
    let unit: AudioUnitParameterUnit
    let address: UInt64  // ← the value of `basekey` after the call
    let dependents: [String]
}


class AlgoBrowser {
    let static NOT_FOUND = UInt64.max
    private let tree: UnsafeRawPointer

    init(tree: UnsafeRawPointer) {
        self.tree = tree
    }

    func firstSet(from key: inout UInt64) -> String? {
        guard let ptr = algoparam_get_first_set(tree, &key) else { return nil }
        return String(cString: ptr)
    }

    func nextSet(from key: inout UInt64) -> String? {
        guard let ptr = algoparam_get_next_set(tree, &key) else { return nil }
        return String(cString: ptr)
    }

    func firstParam(from key: inout UInt64) -> AlgoParam? {
        let raw = algoparam_get_first_param(tree, &key)
        return mapCParam(raw, address: key)
    }

    func nextParam(from key: inout UInt64) -> AlgoParam? {
        let raw = algoparam_get_next_param(tree, &key)
        return mapCParam(raw, address: key)
    }

    private func mapCParam(_ cparam: AlgoCParam, address: UInt64) -> AlgoParam? {
        // Use sentinel to detect invalid result
        guard address != NOT_FOUND {
            return nil
        }

        let key = String(cString: keyPtr)
        let name = String(cString: namePtr)
        let unit = AudioUnitParameterUnit(rawValue: UInt32(cparam.dtype)) ?? .generic

        var dependents: [String] = []
        if let rawList = cparam.dependents {
            var ptr = rawList
            while let item = ptr.pointee {
                dependents.append(String(cString: item))
                ptr = ptr.advanced(by: 1)
            }
        }

        return AlgoParam(
            key: key, 
            name: name, 
            min: cparam.min, 
            max: cparam.max, 
            unit: unit, 
            address: address, 
            dependents: dependents)
    }
}


/// Build a parameter tree including dependencies here...
extension AlgoBrowser {
    func buildParameterTree(from rootKey: UInt64 = NOT_FOUND) -> AUParameterTree {
        var parameterLookup: [String: AUParameter] = [:]
        var paramDependents: [String: [String]] = [:] // Store fully qualified dependent names

        func buildSubtree(from key: UInt64, groupPath: [String]) -> AUParameterNode {
            var groupChildren: [AUParameterNode] = []

            // Traverse child groups
            var setKey = key
            if let setName = firstSet(from: &setKey) {
                groupChildren.append(buildSubtree(from: setKey, groupPath: groupPath + [setName]))
                while let nextSetName = nextSet(from: &setKey) {
                    groupChildren.append(buildSubtree(from: setKey, groupPath: groupPath + [nextSetName]))
                }
            }

            // Traverse parameters
            var paramKey = key
            if let param = firstParam(from: &paramKey) {
                let fqName = (groupPath + [param.key]).joined(separator: "::")
                let node = makeAUParameter(from: param, fqName: fqName, groupPath: groupPath)
                parameterLookup[fqName] = node
                paramDependents[fqName] = qualifyDependents(param.dependents, groupPath: groupPath)
                groupChildren.append(node)

                while let next = nextParam(from: &paramKey) {
                    let fqNext = (groupPath + [next.key]).joined(separator: "::")
                    let node = makeAUParameter(from: next, fqName: fqNext, groupPath: groupPath)
                    parameterLookup[fqNext] = node
                    paramDependents[fqNext] = qualifyDependents(next.dependents, groupPath: groupPath)
                    groupChildren.append(node)
                }
            }

            let groupName = groupPath.last ?? "Root"
            return AUParameterGroup.createGroup(
                withIdentifier: "group_\(key)",
                name: groupName,
                children: groupChildren
            )
        }

        func makeAUParameter(from param: AlgoParam, fqName: String, groupPath: [String]) -> AUParameter {
            return AUParameter(
                identifier: param.key,
                name: param.name,
                address: param.address,
                min: param.min,
                max: param.max,
                unit: param.unit,
                unitName: nil,
                flags: [.flag_IsReadable, .flag_IsWritable],
                valueStrings: nil,
                dependentParameters: nil // Filled later
            )
        }

        func qualifyDependents(_ dependents: [String], groupPath: [String]) -> [String] {
            return dependents.map { dep in
                if dep.contains("::") {
                    // Relative to subtree root
                    return (groupPath + dep.split(separator: ".").map(String.init)).joined(separator: ".")
                } else {
                    // Same group
                    return (groupPath + [dep]).joined(separator: ".")
                }
            }
        }

        let root = buildSubtree(from: rootKey, groupPath: ["Root"])
        let tree = AUParameterTree.createTree(withChildren: [root])

        // Resolve dependent parameters
        for (fqName, depPaths) in paramDependents {
            guard let node = parameterLookup[fqName] else { continue }
            let resolved = depPaths.compactMap { parameterLookup[$0] }
            node.setValue(resolved, forKey: "dependentParameters")
        }

        return tree
    }
}


extension AUParameterTree {
    func prettyPrint() {
        func printNode(_ node: AUParameterNode, indent: Int) {
            let prefix = String(repeating: "  ", count: indent)

            if let param = node as? AUParameter {
                print("\(prefix)↳ Param: \(param.identifier)")
                print("\(prefix)   Name: \(param.displayName)")
                print("\(prefix)   Address: \(param.address)")
                print("\(prefix)   Range: [\(param.min);\(param.max)]")
                print("\(prefix)   Unit: \(param.unit)")

                if let dependents = param.value(forKey: "dependentParameters") as? [AUParameter], !dependents.isEmpty {
                    print("\(prefix)   Affects parameters:")
                    for dep in dependents {
                        print("\(prefix)     - \(dep.identifier) [\(dep.displayName)], addr: \(dep.address)")
                    }
                }
            } else {
                print("\(prefix)Group: \(node.displayName)")
                for child in node.children {
                    printNode(child, indent: indent + 1)
                }
            }
        }

        for root in children {
            printNode(root, indent: 0)
        }
    }
}

