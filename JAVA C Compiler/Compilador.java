
import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class Compilador {

    // Expresiones regulares
    private static final String IDENTIFIER_REGEX = "[a-zA-Z_][a-zA-Z0-9_]{0,30}";
    private static final String NUMBER_REGEX = "[0-9]+";
    private static final String ASSIGNMENT_REGEX = "=";
    private static final String LEFT_PAREN_REGEX = "\\(";
    private static final String RIGHT_PAREN_REGEX = "\\)";
    private static final String LEFT_BRACE_REGEX = "\\{";
    private static final String RIGHT_BRACE_REGEX = "\\}";
    private static final String LEFT_BRACKET_REGEX = "\\[";
    private static final String RIGHT_BRACKET_REGEX = "\\]";
    private static final String COMA_REGEX = ",";
    private static final String DOT_REGEX = ".";
    private static final String SEMICOLON_REGEX = ";";
    private static final String COLON_REGEX = ":";
    private static final String STRING_REGEX = "\"(([^\"\\\\]|\\\\.)*)\""; // Soporte para cadenas
    private static final String CHAR_REGEX = "'([^'\\\\]|\\\\.){1}'"; // Soporte para un solo caracter
    private static final String SINGLE_LINE_COMMENT_REGEX = "\\/\\/.*"; // Soporte para comentarios unilinea
    private static final String MULTI_LINE_COMMENT_REGEX = "/\\*([^*]|\\*(?!/))*\\*/"; //Soporte para comentarios multilinea
    private static final String COMBINED_ASSIGNMENT_REGEX = "(\\+=|-=|\\*=|/=)";
    private static final String INCREMENT_REGEX = "(\\+\\+|--)";
    private static final String OPERATOR_REGEX = "[+\\-*/%^!]+";
    private static final String LOGIC_OPERATOR_REGEX = "(&&|\\|\\|)";
    private static final String BIT_OPERATOR_REGEX = "(&|\\||\\^|~|<<|>>)";
    private static final String COMPARATOR_REGEX = "(==|!=|<|<=|>|>=|<>)"; // Ajustado para comparadores
    private static final String RESERVED_WORDS_REGEX = "(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)";
    private static final String PRIMITIVE_TYPES_REGEX = "(int|char|float|double|void)";
    private static final String DO_REGEX = "(do)";
    private static final String LOOPS_REGEX = "(while|for)";
    private static final String CONDITIONS_REGEX = "(if|else|switch)";
    private static final String CASE_REGEX = "(case)";

    private static final String INITIALIZE_REGEX = "\\|DT\\|\\|ID\\|(\\|AS\\|(\\|NU\\||\\|ID\\|)(\\|OP\\|(\\|ID\\||\\|NU\\|))*){0,1}(\\|COMA\\|\\|ID\\|(\\|AS\\|(\\|NU\\||\\|ID\\|)(\\|OP\\|(\\|ID\\||\\|NU\\|))*){0,1})*\\|SC\\|"; //WithOut Array Support
    private static final String ID_ASSIGNMENT_REGEX = "\\|ID\\|((\\|AS\\||\\|AC\\|)(\\|NU\\||\\|ID\\|)(\\|OP\\|(\\|ID\\||\\|NU\\|))*)\\|SC\\|"; //Only simple int support
    private static final String LOOP_REGEX = "\\|LOOP\\|\\|LP\\|((\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|)(\\|LO\\|(\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|))*){0,1}\\|RP\\|\\|LB\\|";
    private static final String CONDITION_REGEX = "\\|CON\\|\\|LP\\|((\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|)(\\|LO\\|(\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|))*){0,1}\\|RP\\|\\|LB\\|";
    private static final String FUNCTION_REGEX = "\\|DT\\|\\|ID\\|\\|LP\\|(\\|DT\\|\\|ID\\|(\\|COMA\\|\\|DT\\|\\|ID\\|)*){0,1}\\|RP\\|\\|LB\\|";
    private static final String END_REGEX = "\\|RB\\|";
    private static final String DO_LOOP_REGEX = "\\|DO\\|\\|LB\\|";
    private static final String END_DO_REGEX = "\\|LOOP\\|\\|LP\\|((\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|)(\\|LO\\|(\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|))*){0,1}\\|RP\\|\\|SC\\|";


    public static boolean error;
    public static String errorCont;
    public static int linea = 1;

    public static void main(String[] args) {
        // Ruta del archivo .txt
        String filePath = "prueba.txt";

        try (BufferedReader br = new BufferedReader(new FileReader(filePath))) {
            StringBuilder contentBuilder = new StringBuilder();
            String line;

            while ((line = br.readLine()) != null) {
                line = handleComments(line);
                contentBuilder.append(line).append(" "); // Unir las líneas en una sola
            }

            String content = contentBuilder.toString();
            String[] logicalLines = content.split("(?<=[;{}])"); // Divide cada línea lógica por ';'

            System.out.println("");
            System.out.println("");

            for (String logicalLine : logicalLines) {

                logicalLine = logicalLine.trim(); // Quitar espacios
                if (!logicalLine.isEmpty()) {

                    String primeLine = logicalLine;

                    String auxLine = handleComments(logicalLine);
                    logicalLine = auxLine;

                    // Aplica la función que maneja cadenas y caracteres
                    auxLine = handleStringsAndChars(logicalLine);
                    logicalLine = auxLine;

                    // Paso 2: Tokeniza la línea normalmente
                    String[] tokens = logicalLine.split("(?<=\\W)|(?=\\W)"); // Separar por símbolos no alfanuméricos

                    StringBuffer lineBuffer = new StringBuffer();

                    System.out.println("");
                    System.out.println("Analizando línea: " + primeLine);
                    boolean esValido = false;
                    System.out.print(">>");

                    for (String token : tokens) {

                        token = token.trim();

                        if (token.isEmpty()) {
                            continue; // Ignorar tokens vacíos
                        }
                        if (isPrimitiveDataType(token)) {
                            lineBuffer.append("|DT|");
                            esValido = true;
                        } else if (isCondition(token)) {
                            lineBuffer.append("|CON|");
                            esValido = true;
                        } else if (isCase(token)) {
                            lineBuffer.append("|CASE|");
                            esValido = true;
                        } else if (isDo(token)) {
                            lineBuffer.append("|DO|");
                            esValido = true;
                        } else if (isLoop(token)) {
                            lineBuffer.append("|LOOP|");
                            esValido = true;
                        } else if (isReservedWord(token)) {
                            lineBuffer.append("|PR|");
                            esValido = true;
                        } else if (isString(token)) {
                            lineBuffer.append("|STR|");
                            esValido = true;
                        } else if (isChar(token)) {
                            lineBuffer.append("|CHR|"); // Token reconocido como carácter
                            esValido = true;
                        } else if (isIdentifier(token)) {
                            lineBuffer.append("|ID|");
                            esValido = true;
                        } else if (isIncrement(token)) {
                            lineBuffer.append("|IC|");
                            esValido = true;
                        } else if (isCombinedAssignment(token)) {
                            lineBuffer.append("|AC|");
                            esValido = true;
                        } else if (isAssignment(token)) {
                            lineBuffer.append("|AS|");
                            esValido = true;
                        } else if (isDigit(token)) {
                            lineBuffer.append("|NU|");
                            esValido = true;
                        } else if (isOperator(token)) {
                            lineBuffer.append("|OP|");
                            esValido = true;
                        } else if (isLogicOperator(token)) {
                            lineBuffer.append("|LO|");
                            esValido = true;
                        } else if (isBitOperator(token)) {
                            lineBuffer.append("|BO|");
                            esValido = true;
                        } else if (isComparator(token)) {
                            lineBuffer.append("|CO|");
                            esValido = true;
                        } else if (isLeftParen(token)) {
                            lineBuffer.append("|LP|");
                            esValido = true;
                        } else if (isRightParen(token)) {
                            lineBuffer.append("|RP|");
                            esValido = true;
                        } else if (isLeftBrace(token)) {
                            lineBuffer.append("|LB|");
                            esValido = true;
                        } else if (isRightBrace(token)) {
                            lineBuffer.append("|RB|");
                            esValido = true;
                        } else if (isLeftBracket(token)) {
                            lineBuffer.append("|LBR|");
                            esValido = true;
                        } else if (isRightBracket(token)) {
                            lineBuffer.append("|RBR|");
                            esValido = true;
                        } else if (isComa(token)) {
                            lineBuffer.append("|COMA|");
                            esValido = true;
                        } else if (isColon(token)) {
                            lineBuffer.append("|COL|");
                            esValido = true;
                        } else if (isSemicolon(token)) {
                            lineBuffer.append("|SC|");
                            esValido = true;
                        } else if (isDot(token)) {
                            lineBuffer.append("|DOT|");
                            esValido = true;
                        } else {
                            errorCont = " Token no reconocido: " + token + ".";
                            throw new compileException("Error en la linea:" + linea + errorCont);
                        }
                    }

                    if (isInitial(lineBuffer.toString())) {
                        esValido = true;
                    } else if (isIDAssignment(lineBuffer.toString())){
                        esValido = true;
                    } else if (isLineLoop(lineBuffer.toString())){
                        esValido = true;
                    } else if (isLineCondition(lineBuffer.toString())){
                        esValido = true;
                    } else if (isFunction(lineBuffer.toString())){
                        esValido = true;
                    } else if (isEnd(lineBuffer.toString())){
                        esValido = true;
                    } else if (isDoLoop(lineBuffer.toString())){
                        esValido = true;
                    } else if (isEndDo(lineBuffer.toString())){
                        esValido = true;
                    } else {
                        errorCont = "Sintaxis Invalida.";
                        throw new compileException("Error: "+ errorCont +" En la linea:" + linea);
                    }
                }

                linea++;

                System.out.println("Linea Valida.");

            }           

        } catch (IOException e) {
            e.getMessage();
            e.printStackTrace();
        } catch (compileException e) {
            e.getMessage();
            e.printStackTrace();
        }
    }

    // Paso 1: Reemplaza las cadenas y caracteres por marcadores
    public static String handleStringsAndChars(String line) {

        String newLine = line;
        Pattern stringPattern = Pattern.compile(STRING_REGEX);
        Pattern charPattern = Pattern.compile(CHAR_REGEX);

        // Reemplazar cadenas por "STR"
        Matcher stringMatcher = stringPattern.matcher(newLine);
        newLine = stringMatcher.replaceAll(" STR ");

        // Reemplazar caracteres por "CHR"
        Matcher charMatcher = charPattern.matcher(newLine);
        newLine = charMatcher.replaceAll(" CHR ");

        return newLine;
    }

    // Función para manejar comentarios
    public static String handleComments(String line) {

        String newLine = line;
        Pattern singleLinePattern = Pattern.compile(SINGLE_LINE_COMMENT_REGEX);
        Pattern multiLinePattern = Pattern.compile(MULTI_LINE_COMMENT_REGEX);

        // Reemplazar comentarios de una línea por "COMMENT"
        Matcher singleLineMatcher = singleLinePattern.matcher(newLine);
        newLine = singleLineMatcher.replaceAll("");

        // Reemplazar comentarios de varias líneas por "COMMENT"
        Matcher multiLineMatcher = multiLinePattern.matcher(newLine);
        line = multiLineMatcher.replaceAll("");

        return line;
    }

    // Funciones para reconocer diferentes tipos de tokens
    public static boolean isPrimitiveDataType(String input) {
        return Pattern.matches(PRIMITIVE_TYPES_REGEX, input);
    }

    public static boolean isDo(String input) {
        return Pattern.matches(DO_REGEX, input);
    }
    
    public static boolean isLoop(String input) {
        return Pattern.matches(LOOPS_REGEX, input);
    }

    public static boolean isCondition(String input) {
        return Pattern.matches(CONDITIONS_REGEX, input);
    }

    public static boolean isCase(String input) {
        return Pattern.matches(CASE_REGEX, input);
    }

    public static boolean isIdentifier(String input) {
        return Pattern.matches(IDENTIFIER_REGEX, input);
    }

    public static boolean isDigit(String input) {
        return Pattern.matches(NUMBER_REGEX, input);
    }

    public static boolean isAssignment(String input) {
        return Pattern.matches(ASSIGNMENT_REGEX, input);
    }

    public static boolean isCombinedAssignment(String input) {
        return Pattern.matches(COMBINED_ASSIGNMENT_REGEX, input);
    }

    public static boolean isIncrement(String input) {
        return Pattern.matches(INCREMENT_REGEX, input);
    }

    public static boolean isOperator(String input) {
        return Pattern.matches(OPERATOR_REGEX, input);
    }

    public static boolean isLogicOperator(String input) {
        return Pattern.matches(LOGIC_OPERATOR_REGEX, input);
    }

    public static boolean isBitOperator(String input) {
        return Pattern.matches(BIT_OPERATOR_REGEX, input);
    }

    public static boolean isComparator(String input) {
        return Pattern.matches(COMPARATOR_REGEX, input);
    }

    public static boolean isReservedWord(String input) {
        return Pattern.matches(RESERVED_WORDS_REGEX, input);
    }

    public static boolean isString(String input) {
        return input.equals("STR");
    }

    public static boolean isChar(String input) {
        return input.equals("CHR"); 
    }        

    public static boolean isLeftParen(String input) {
        return Pattern.matches(LEFT_PAREN_REGEX, input);
    }

    public static boolean isRightParen(String input) {
        return Pattern.matches(RIGHT_PAREN_REGEX, input);
    }

    public static boolean isLeftBrace(String input) {
        return Pattern.matches(LEFT_BRACE_REGEX, input);
    }

    public static boolean isRightBrace(String input) {
        return Pattern.matches(RIGHT_BRACE_REGEX, input);
    }

    public static boolean isLeftBracket(String input) {
        return Pattern.matches(LEFT_BRACKET_REGEX, input);
    }

    public static boolean isRightBracket(String input) {
        return Pattern.matches(RIGHT_BRACKET_REGEX, input);
    }

    public static boolean isComa(String input) {
        return Pattern.matches(COMA_REGEX, input);
    }

    public static boolean isDot(String input) {
        return Pattern.matches(DOT_REGEX, input);
    }

    public static boolean isColon(String input) {
        return Pattern.matches(COLON_REGEX, input);
    }

    public static boolean isSemicolon(String input) {
        return Pattern.matches(SEMICOLON_REGEX, input);
    }

//-------------------------------------------------------------------------------------------------

    public static boolean isInitial(String input){
        return Pattern.matches(INITIALIZE_REGEX, input);
    }

    public static boolean isIDAssignment(String input){
        return Pattern.matches(ID_ASSIGNMENT_REGEX, input);
    }

    public static boolean isLineLoop(String input){
        return Pattern.matches(LOOP_REGEX, input);
    }

    public static boolean isLineCondition(String input){
        return Pattern.matches(CONDITION_REGEX, input);
    }

    public static boolean isFunction(String input){
        return Pattern.matches(FUNCTION_REGEX, input);
    }

    public static boolean isEnd(String input){
        return Pattern.matches(END_REGEX, input);
    }

    public static boolean isDoLoop(String input){
        return Pattern.matches(DO_LOOP_REGEX, input);
    }

    public static boolean isEndDo(String input){
        return Pattern.matches(END_DO_REGEX, input);
    }
}
