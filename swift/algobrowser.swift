import Foundation
import AudioToolbox

// Native Swift representation of the Rust-provided parameter
class AlgoParam {
    let key: String
    let name: String
    let min: Float
    let max: Float
    let unit: AudioUnitParameterUnit
    let address: UInt64  // ← the value of `basekey` after the call
    let dependents: [String]
    var dependentAddresses: [UInt64]
    
    init(key: String, name: String, min: Float, max: Float, unit: AudioUnitParameterUnit, address: UInt64, dependents: [String], dependentAddresses: [UInt64]) {
        self.key = key
        self.name = name
        self.min = min
        self.max = max
        self.unit = unit
        self.address = address
        self.dependents = dependents
        self.dependentAddresses = dependentAddresses
    }
    
    func asAUParameter() -> AUParameter {
        AUParameterTree.createParameter(withIdentifier: key, name: name, address: address, min: min, max: max, unit: unit, unitName: nil, valueStrings: nil, dependentParameters: dependentAddresses.map( { NSNumber.init(value:$0) }))
    }
}

class AlgoParamSet {
    let key: String
    let name: String
    let address: UInt64
    var parameters: [AlgoParam]
    var children: [AlgoParamSet]
    
    init(key: String, name: String, address: UInt64, parameters: [AlgoParam], children: [AlgoParamSet]) {
        self.key = key
        self.name = name
        self.address = address
        self.parameters = parameters
        self.children = children
    }
    
    func parameter(for keyPath: String) -> AlgoParam? {
        let pathelems = keyPath.split(separator: ".", maxSplits: 1)
        if pathelems.count == 1 {
            // We are at where the child should be
            return parameters.first { $0.key == String(pathelems[0]) }
        } else {
            return children.first { $0.key == String(pathelems[0]) }?.parameter(for: String(pathelems[1]))
        }
    }
    
    func asAUParameterGroup() -> AUParameterGroup {
        let params = parameters.map { $0.asAUParameter() }
        let groups = children.map { $0.asAUParameterGroup() }
        let bunch = params + groups
        return AUParameterTree.createGroup(withIdentifier: key, name: name, children: bunch)
    }
}

let NOT_FOUND = UInt64.max

class AlgoBrowser {
    private let tree: UnsafeRawPointer

    init(tree: UnsafeRawPointer) {
        self.tree = tree
    }

    private func firstSet(from key: inout UInt64) -> AlgoParamSet? {
        let raw = algoparam_get_first_set(tree, &key)
        return mapCParamSet(raw, address: key)
    }

    private func nextSet(from key: inout UInt64) -> AlgoParamSet? {
        let raw = algoparam_get_next_set(tree, &key)
        return mapCParamSet(raw, address: key)
    }

    private func firstParam(from key: inout UInt64) -> AlgoParam? {
        let raw = algoparam_get_first_param(tree, &key)
        return mapCParam(raw, address: key)
    }

    private func nextParam(from key: inout UInt64) -> AlgoParam? {
        let raw = algoparam_get_next_param(tree, &key)
        return mapCParam(raw, address: key)
    }

    private func mapCParam(_ cparam: AlgoCParam, address: UInt64) -> AlgoParam? {
        // Use sentinel to detect invalid result
        guard address != NOT_FOUND else {
            return nil
        }

        let key = String(cString: cparam.key)
        let name = String(cString: cparam.name)
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
            dependents: dependents,
            dependentAddresses: []
        )
    }
    
    private func mapCParamSet(_ cparamset: AlgoCParamSet, address: UInt64) -> AlgoParamSet? {
        guard address != NOT_FOUND else {
            return nil
        }
        
        let key = String(cString: cparamset.key)
        let name = String(cString: cparamset.name)
        
        return AlgoParamSet(key: key, name: name, address: address, parameters: [], children: [])
    }
    
    private func fixupDependencies(in algoparamset: AlgoParamSet)  {
        // Iterate over all parameters
        for param in algoparamset.parameters {
            // ... and all dependents
            for dependent in param.dependents {
                // Find the address of the parameters that are described here
                if let address = algoparamset.parameter(for: dependent)?.address {
                    param.dependentAddresses.append(address)
                }
            }
        }
        for child in algoparamset.children {
            fixupDependencies(in: child)
        }
    }
    
    private func build(from root: UInt64, basekey: String, displayname: String) -> AlgoParamSet {
        var rootParam = root
        let set = AlgoParamSet(key: basekey, name: displayname, address: root, parameters: [], children: [])
        // Get all parameters on this level
        var param = firstParam(from: &rootParam)
        while rootParam != NOT_FOUND {
            set.parameters.append(param!)
            param = nextParam(from: &rootParam)
        }
        // Get all child nodes
        rootParam = root
        var setChild = firstSet(from: &rootParam)
        while rootParam != NOT_FOUND {
            set.children.append(build(from: rootParam, basekey: setChild!.key, displayname: setChild!.name))
            setChild = nextSet(from: &rootParam)
        }
        return set
    }
    
    private func build() -> AlgoParamSet {
        let set = build(from: NOT_FOUND, basekey: "root", displayname: "Root")
        fixupDependencies(in: set)
        return set
    }
    
    func getAUParameterTree() -> AUParameterTree {
        
        let root = build().asAUParameterGroup()
        return AUParameterTree.createTree(withChildren: [root])
    }
}

