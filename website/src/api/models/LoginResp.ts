/* tslint:disable */
/* eslint-disable */
/**
 * server
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * Contact: booiris02@gmail.com
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { mapValues } from '../runtime';
/**
 * 
 * @export
 * @interface LoginResp
 */
export interface LoginResp {
    /**
     * 
     * @type {number}
     * @memberof LoginResp
     */
    code: number;
    /**
     * 
     * @type {string}
     * @memberof LoginResp
     */
    message: string;
    /**
     * 
     * @type {string}
     * @memberof LoginResp
     */
    jwt: string;
}

/**
 * Check if a given object implements the LoginResp interface.
 */
export function instanceOfLoginResp(value: object): value is LoginResp {
    if (!('code' in value) || value['code'] === undefined) return false;
    if (!('message' in value) || value['message'] === undefined) return false;
    if (!('jwt' in value) || value['jwt'] === undefined) return false;
    return true;
}

export function LoginRespFromJSON(json: any): LoginResp {
    return LoginRespFromJSONTyped(json, false);
}

export function LoginRespFromJSONTyped(json: any, ignoreDiscriminator: boolean): LoginResp {
    if (json == null) {
        return json;
    }
    return {
        
        'code': json['code'],
        'message': json['message'],
        'jwt': json['jwt'],
    };
}

export function LoginRespToJSON(value?: LoginResp | null): any {
    if (value == null) {
        return value;
    }
    return {
        
        'code': value['code'],
        'message': value['message'],
        'jwt': value['jwt'],
    };
}

